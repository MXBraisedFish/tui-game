use std::io::{self, Stdout};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};
use serde::Deserialize;

use tui_game::app;
use tui_game::app::content_cache;
use tui_game::app::game_selection::{GameSelection, GameSelectionAction};
use tui_game::app::i18n;
use tui_game::app::layout::{MENU_MIN_HEIGHT, MENU_MIN_WIDTH};
use tui_game::app::menu::{Menu, MenuAction};
use tui_game::app::placeholder_pages::{self, PlaceholderPage};
use tui_game::app::settings;
use tui_game::core::runtime::{LaunchMode, launch_game};
use tui_game::core::save;
use tui_game::game::registry::GameDescriptor;
use tui_game::terminal::renderer;
use tui_game::terminal::size_watcher;

const RUNTIME_VERSION: &str = env!("CARGO_PKG_VERSION");
const LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest";
const MAX_UI_EVENTS_PER_FRAME: usize = 256;
const ACTIVE_FRAME_BUDGET: Duration = Duration::from_millis(16);
const IDLE_FRAME_BUDGET: Duration = Duration::from_millis(250);
const UI_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Deserialize)]
struct LatestReleaseResponse {
    tag_name: String,
}

/// Top-level application state used by the main loop.
pub enum AppState {
    /// Main menu page.
    MainMenu {
        menu: Menu,
    },
    GameSelection {
        ui: GameSelection,
    },
    Settings {
        ui: settings::SettingsState,
    },
    About,
    Continue,
    /// Program is preparing to exit.
    Exiting,
}

struct PendingNewGameStart {
    target_game: GameDescriptor,
    saved_game_name: String,
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut out = io::stdout();
        execute!(out, EnterAlternateScreen, Hide)?;
        let backend = CrosstermBackend::new(out);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), Show, LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

fn install_panic_hook() {
    let old = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let mut out = io::stdout();
        let _ = execute!(out, Show, LeaveAlternateScreen);
        tui_game::utils::host_log::append_host_error(
            "host.error.program_crashed",
            &[("panic_info", &panic_info.to_string())],
        );
        old(panic_info);
    }));
}

fn main() {
    if let Err(err) = run() {
        let err_text = format!("{err:#}");
        tui_game::utils::host_log::append_host_error("host.error.raw", &[("err", &err_text)]);
    }
}

/// Initialize subsystems and drive the application main loop.
fn run() -> Result<()> {
    if handle_cli_passthrough()? {
        return Ok(());
    }

    install_panic_hook();
    cleanup_legacy_runtime_data()?;
    i18n::init("us-en")?;
    initialize_runtime_layout()?;

    // Initialize terminal session.
    let mut session = TerminalSession::new()?;
    render_loading_screen(
        &mut session.terminal,
        &content_cache::LoadingProgress {
            percent: 0,
            message: i18n::t_or("loading.startup.preparing", "Preparing startup..."),
        },
    )?;
    content_cache::reload_with_progress(|progress| {
        let _ = render_loading_screen(
            &mut session.terminal,
            &content_cache::LoadingProgress {
                percent: progress.percent,
                message: progress.message,
            },
        );
    });
    let runtime_version = normalized_tag(RUNTIME_VERSION);
    let update_check_rx = spawn_update_check(runtime_version.clone());
    let mut update_hint: Option<String> = None;
    let mut state = AppState::MainMenu { menu: Menu::new() };
    let mut pending_new_game_start: Option<PendingNewGameStart> = None;
    let mut force_ui_full_redraw = false;
    let mut last_activity_at = Instant::now();
    let mut ui_dirty = true;
    let mut last_size_ok: Option<bool> = None;

    loop {
        let frame_start = Instant::now();
        let idle_mode = frame_start.duration_since(last_activity_at) >= UI_IDLE_TIMEOUT;
        let frame_budget = if idle_mode {
            IDLE_FRAME_BUDGET
        } else {
            ACTIVE_FRAME_BUDGET
        };

        if let AppState::MainMenu { menu } = &mut state {
            sync_continue_item(menu);
        }

        if let AppState::Settings { ui } = &mut state
            && settings::poll_mod_hot_reload(ui)
        {
            ui_dirty = true;
            force_ui_full_redraw = true;
        }
        if let AppState::GameSelection { ui } = &mut state
            && ui.poll_mod_hot_reload()
        {
            ui_dirty = true;
            force_ui_full_redraw = true;
        }

        // Try to receive the background version check result.
        if update_hint.is_none() {
            if let Ok(Some(latest_tag)) = update_check_rx.try_recv() {
                update_hint = Some(latest_tag);
                ui_dirty = true;
            }
        }

        let (min_width, min_height) = minimum_size_for_state(&state);
        let size_state = size_watcher::check_size(min_width, min_height)?;

        let initial_poll_timeout = if should_keep_ui_animating(&state) || !idle_mode {
            Duration::from_millis(0)
        } else {
            frame_budget
        };
        let mut handled_events = 0usize;
        let mut saw_input_event = false;
        let mut saw_resize_event = false;
        let mut polled = event::poll(initial_poll_timeout)?;
        while handled_events < MAX_UI_EVENTS_PER_FRAME && polled {
            handled_events += 1;
            match event::read()? {
                Event::Key(key) => {
                    saw_input_event = true;
                    if !size_state.size_ok
                        && matches!(key.kind, KeyEventKind::Press)
                        && matches!(
                            key.code,
                            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q')
                        )
                    {
                        state = AppState::Exiting;
                        break;
                    }
                    handle_key_event(
                        &mut state,
                        &mut pending_new_game_start,
                        &mut force_ui_full_redraw,
                        key,
                    )?;
                }
                Event::Resize(_, _) => {
                    saw_resize_event = true;
                    force_ui_full_redraw = true;
                }
                _ => {}
            }
            polled = event::poll(Duration::from_millis(0))?;
        }

        if saw_input_event || saw_resize_event {
            last_activity_at = Instant::now();
            ui_dirty = true;
        }

        if matches!(state, AppState::Exiting) {
            break;
        }

        let should_draw = force_ui_full_redraw
            || ui_dirty
            || last_size_ok != Some(size_state.size_ok)
            || should_keep_ui_animating(&state);

        if size_state.size_ok {
            if force_ui_full_redraw {
                session.terminal.clear()?;
                force_ui_full_redraw = false;
            }
            if should_draw {
                session.terminal.draw(|frame| match &mut state {
                    AppState::MainMenu { menu } => {
                        app::menu::render_main_menu(
                            frame,
                            menu,
                            RUNTIME_VERSION,
                            update_hint.as_deref(),
                        );
                    }
                    AppState::GameSelection { ui } => {
                        if let Some(pending) = pending_new_game_start.as_ref() {
                            render_new_game_confirm(frame, &pending.saved_game_name);
                        } else {
                            ui.render(frame, frame.area());
                        }
                    }
                    AppState::Settings { ui } => {
                        settings::render(frame, ui);
                    }
                    AppState::About => {
                        placeholder_pages::render_placeholder(
                            frame,
                            PlaceholderPage::About,
                            runtime_version.as_str(),
                            None,
                        );
                    }
                    AppState::Continue => {
                        placeholder_pages::render_placeholder(
                            frame,
                            PlaceholderPage::Continue,
                            runtime_version.as_str(),
                            None,
                        );
                    }
                    AppState::Exiting => {}
                })?;
                ui_dirty = false;
                last_size_ok = Some(true);
            }
        } else {
            if should_draw {
                size_watcher::draw_size_warning(&size_state, min_width, min_height)?;
                ui_dirty = false;
                last_size_ok = Some(false);
            }
        }

        let elapsed = frame_start.elapsed();
        if elapsed < frame_budget {
            thread::sleep(frame_budget - elapsed);
        }
    }

    drop(session);

    Ok(())
}

fn render_loading_screen(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    progress: &content_cache::LoadingProgress,
) -> Result<()> {
    let title = i18n::t_or("loading.startup.title", "Loading data");
    let hint = i18n::t_or(
        "loading.startup.hint",
        "The program is preparing game and mod resources. This is not a freeze.",
    );
    terminal.draw(|frame| {
        let area = frame.area();
        let block_width = area.width.saturating_sub(2).max(1);
        let hint_lines = estimate_wrapped_lines(&hint, block_width.saturating_sub(2));
        let block_height = 5u16.saturating_add(hint_lines.max(1));
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(block_height),
                Constraint::Min(0),
            ])
            .split(area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(title.clone(), Style::default().fg(Color::White)),
                Span::styled(" ", Style::default()),
            ]))
            .border_style(Style::default().fg(Color::White));
        let inner = block.inner(layout[1]);
        frame.render_widget(block, layout[1]);

        let body = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(hint_lines.max(1)),
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("{}%", progress.percent),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center),
            body[0],
        );

        frame.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(Color::LightGreen).bg(Color::DarkGray))
                .percent(progress.percent.min(100))
                .label(""),
            body[1],
        );

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                progress.message.clone(),
                Style::default().fg(Color::White),
            )))
            .alignment(Alignment::Center),
            body[2],
        );

        frame.render_widget(
            Paragraph::new(hint.clone())
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false })
                .style(Style::default().fg(Color::DarkGray)),
            body[3],
        );
    })?;
    Ok(())
}

fn estimate_wrapped_lines(text: &str, width: u16) -> u16 {
    let width = width.max(1) as usize;
    let mut total = 0usize;
    for raw_line in text.lines() {
        let len = raw_line.chars().count().max(1);
        total += len.div_ceil(width);
    }
    total.max(1) as u16
}

fn should_keep_ui_animating(state: &AppState) -> bool {
    match state {
        AppState::Settings { ui } => {
            ui.mod_safe_dialog.is_some()
                || ui.cleanup_dialog.is_some()
                || ui.default_safe_mode_disable_dialog.is_some()
                || ui.security_success_at.is_some()
        }
        _ => false,
    }
}

fn initialize_runtime_layout() -> Result<()> {
    let app_data = tui_game::utils::path_utils::app_data_dir()?;
    std::fs::create_dir_all(app_data.join("mod"))?;
    std::fs::create_dir_all(app_data.join("official"))?;
    std::fs::create_dir_all(app_data.join("cache"))?;
    std::fs::create_dir_all(app_data.join("mod_save"))?;
    std::fs::create_dir_all(app_data.join("log"))?;

    let language = tui_game::utils::path_utils::language_file()?;
    if !language.exists() {
        std::fs::write(&language, format!("{}\n", i18n::current_language_code()))?;
    }

    let best_scores = tui_game::utils::path_utils::best_scores_file()?;
    if !best_scores.exists() {
        std::fs::write(&best_scores, "{}\n")?;
    }

    let saves = tui_game::utils::path_utils::saves_file()?;
    if !saves.exists() {
        std::fs::write(&saves, "{\n  \"continue\": {},\n  \"data\": {}\n}\n")?;
    }

    let updater_cache = tui_game::utils::path_utils::updater_cache_file()?;
    if !updater_cache.exists() {
        std::fs::write(&updater_cache, "{}\n")?;
    }

    let _ = tui_game::utils::path_utils::official_games_dir()?;
    Ok(())
}

fn cleanup_legacy_runtime_data() -> Result<()> {
    let app_data = tui_game::utils::path_utils::app_data_dir()?;
    for file_name in [
        "stats.json",
        "lua_saves.json",
        "runtime_best_scores.json",
        "latest_runtime_save.txt",
        "language_pref.txt",
        "mod_state.json",
        "scan_cache.json",
    ] {
        let path = app_data.join(file_name);
        if path.exists() {
            if let Err(err) = std::fs::remove_file(path) {
                tui_game::utils::host_log::append_host_error(
                    "host.error.clean_old_save_failed",
                    &[("err", &err.to_string())],
                );
            }
        }
    }
    for dir_name in ["runtime_save", "runtime-logs"] {
        let path = app_data.join(dir_name);
        if path.exists() {
            if let Err(err) = std::fs::remove_dir_all(path) {
                tui_game::utils::host_log::append_host_error(
                    "host.error.clean_old_save_failed",
                    &[("err", &err.to_string())],
                );
            }
        }
    }
    Ok(())
}

/// Return the minimum terminal size required by the current page.
fn minimum_size_for_state(state: &AppState) -> (u16, u16) {
    match state {
        AppState::MainMenu { .. } => (MENU_MIN_WIDTH, MENU_MIN_HEIGHT),
        AppState::GameSelection { ui } => ui.minimum_size(),
        AppState::Settings { ui } => settings::minimum_size(ui),
        AppState::About | AppState::Continue => (MENU_MIN_WIDTH, MENU_MIN_HEIGHT),
        AppState::Exiting => (MENU_MIN_WIDTH, MENU_MIN_HEIGHT),
    }
}

/// Handle high-level key events based on the current application state.
fn handle_key_event(
    state: &mut AppState,
    pending_new_game_start: &mut Option<PendingNewGameStart>,
    force_ui_full_redraw: &mut bool,
    key: KeyEvent,
) -> Result<()> {
    if !matches!(key.kind, KeyEventKind::Press) {
        return Ok(());
    }

    if !matches!(state, AppState::GameSelection { .. }) {
        *pending_new_game_start = None;
    }

    match state {
        AppState::MainMenu { menu } => match key.code {
            KeyCode::Up | KeyCode::Char('k') => menu.previous(),
            KeyCode::Down | KeyCode::Char('j') => menu.next(),
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(index) = c
                    .to_digit(10)
                    .map(|v| v as usize)
                    .and_then(|v| v.checked_sub(1))
                {
                    menu.set_selected(index);
                }
            }
            KeyCode::Esc => {
                let _ = menu.select_by_shortcut(KeyCode::Esc);
            }
            KeyCode::Enter => {
                if let Some(action) = menu.selected_action() {
                    if matches!(action, MenuAction::Continue) && !menu.can_continue() {
                        return Ok(());
                    }
                    *state =
                        apply_menu_action(action, menu.continue_game_id(), force_ui_full_redraw);
                }
            }
            _ => {}
        },
        AppState::GameSelection { ui } => {
            if pending_new_game_start.is_some() {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        let pending = pending_new_game_start.take();
                        if let Some(pending) = pending {
                            if let Err(err) = save::clear_active_game_save() {
                                let err_text = format!("{err:#}");
                                tui_game::utils::host_log::append_host_error(
                                    "host.error.clean_old_save_failed",
                                    &[("err", &err_text)],
                                );
                            }
                            if let Err(err) = launch_game(&pending.target_game, LaunchMode::New) {
                                let err_text = format!("{err:#}");
                                tui_game::utils::host_log::append_host_error(
                                    "host.error.run_game_failed",
                                    &[
                                        ("game_id", pending.target_game.id.as_str()),
                                        ("err", &err_text),
                                    ],
                                );
                            }
                            reset_terminal_after_runtime()?;
                            *force_ui_full_redraw = true;
                            let games = content_cache::games();
                            ui.refresh_preserving_selection(games);
                        }
                    }
                    KeyCode::Char('n')
                    | KeyCode::Char('N')
                    | KeyCode::Char('q')
                    | KeyCode::Char('Q')
                    | KeyCode::Esc => {
                        *pending_new_game_start = None;
                    }
                    _ => {}
                }
                return Ok(());
            }

            if let Some(action) = ui.handle_event(key) {
                match action {
                    GameSelectionAction::BackToMenu => {
                        *pending_new_game_start = None;
                        *state = AppState::MainMenu { menu: Menu::new() };
                    }
                    GameSelectionAction::LaunchGame(game) => {
                        if let Some(saved_game_id) = save::latest_saved_game_id() {
                            let saved_game_name = resolve_saved_game_name(&saved_game_id);
                            *pending_new_game_start = Some(PendingNewGameStart {
                                target_game: game,
                                saved_game_name,
                            });
                            return Ok(());
                        }
                        if let Err(err) = save::clear_active_game_save() {
                            let err_text = format!("{err:#}");
                            tui_game::utils::host_log::append_host_error(
                                "host.error.clean_old_save_failed",
                                &[("err", &err_text)],
                            );
                        }
                        if let Err(err) = launch_game(&game, LaunchMode::New) {
                            let err_text = format!("{err:#}");
                            tui_game::utils::host_log::append_host_error(
                                "host.error.run_game_failed",
                                &[("game_id", game.id.as_str()), ("err", &err_text)],
                            );
                        }
                        reset_terminal_after_runtime()?;
                        *force_ui_full_redraw = true;
                        let games = content_cache::games();
                        ui.refresh_preserving_selection(games);
                    }
                }
            }
        }

        AppState::Settings { ui } => match settings::handle_key(ui, key.code) {
            settings::SettingsAction::None => {}
            settings::SettingsAction::BackToMenu => {
                *state = AppState::MainMenu { menu: Menu::new() };
            }
        },

        AppState::About | AppState::Continue => match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                *state = AppState::MainMenu { menu: Menu::new() }
            }
            _ => {}
        },

        AppState::Exiting => {}
    }

    Ok(())
}

fn render_new_game_confirm(frame: &mut ratatui::Frame<'_>, saved_game_name: &str) {
    use ratatui::layout::{Alignment, Constraint, Direction, Layout};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Clear, Paragraph, Wrap};

    // Use a full-screen clear as the modal background.
    let area = frame.area();
    frame.render_widget(Clear, area);

    let template = i18n::t("confirm.new_game_overwrite");
    let msg = if template.contains("{game}") {
        template.replace("{game}", saved_game_name)
    } else {
        format!("{template} {saved_game_name}")
    };

    let yes = i18n::t("confirm.new_game_yes");
    let no = i18n::t("confirm.new_game_no");

    // Vertically center the confirmation content.
    let center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

    let p = Paragraph::new(vec![
        Line::from(Span::styled(
            msg,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("{yes}  {no}"),
            Style::default().fg(Color::White),
        )),
    ])
    .style(Style::default().bg(Color::Black))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: false });
    frame.render_widget(p, center[1]);
}

/// Convert a main menu action into the next application state.
fn apply_menu_action(
    action: MenuAction,
    continue_game_id: Option<&str>,
    force_ui_full_redraw: &mut bool,
) -> AppState {
    match action {
        MenuAction::Play => {
            AppState::GameSelection {
                ui: GameSelection::new(content_cache::games()),
            }
        }

        MenuAction::Continue => {
            if let Some(game_id) = continue_game_id {
                let game = content_cache::games()
                    .into_iter()
                    .find(|g| g.id.eq_ignore_ascii_case(game_id));
                if let Some(game) = game {
                    if let Err(err) = launch_game(&game, LaunchMode::Continue) {
                        let err_text = format!("{err:#}");
                        tui_game::utils::host_log::append_host_error(
                            "host.error.continue_game_failed",
                            &[("game_id", game.id.as_str()), ("err", &err_text)],
                        );
                    }
                    let _ = reset_terminal_after_runtime();
                    *force_ui_full_redraw = true;
                }
            }
            AppState::GameSelection {
                ui: GameSelection::new(content_cache::games()),
            }
        }

        MenuAction::Settings => AppState::Settings {
            ui: settings::SettingsState::new(),
        },

        MenuAction::About => AppState::About,

        MenuAction::Quit => AppState::Exiting,
    }
}

fn reset_terminal_after_runtime() -> Result<()> {
    renderer::invalidate_canvas_cache();
    let mut out = io::stdout();
    execute!(out, Clear(ClearType::All), crossterm::cursor::MoveTo(0, 0))?;
    Ok(())
}

/// Handle simple CLI passthrough flags used by wrapper scripts.
fn handle_cli_passthrough() -> Result<bool> {
    let arg = match std::env::args().nth(1) {
        Some(v) => v,
        None => return Ok(false),
    };
    if arg.eq_ignore_ascii_case("--runtime-version") || arg.eq_ignore_ascii_case("-runtime-version")
    {
        println!("v{}", RUNTIME_VERSION.trim());
        return Ok(true);
    }
    Ok(false)
}

fn sync_continue_item(menu: &mut Menu) {
    let Some(game_id) = save::latest_saved_game_id() else {
        menu.set_continue_target(None, None);
        return;
    };

    match resolve_continue_target(&game_id) {
        Some((resolved_id, resolved_name)) => {
            menu.set_continue_target(Some(resolved_id), Some(resolved_name));
        }
        None => {
            let _ = save::clear_active_game_save();
            menu.set_continue_target(None, None);
        }
    }
}

fn resolve_saved_game_name(game_id: &str) -> String {
    if let Some(game) = content_cache::find_game(game_id) {
        return game.display_name;
    }
    "--".to_string()
}

fn resolve_continue_target(game_id: &str) -> Option<(String, String)> {
    let game = content_cache::find_game(game_id)?;

    if !save::game_has_continue_save(game_id) {
        return None;
    }

    Some((game.id, game.display_name))
}

fn normalized_tag(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with('v') || trimmed.starts_with('V') {
        format!("v{}", trimmed[1..].trim())
    } else {
        format!("v{}", trimmed)
    }
}

/// Start the background update check thread.
fn spawn_update_check(current_version: String) -> Receiver<Option<String>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = check_latest_release(&current_version).ok().flatten();
        let _ = tx.send(result);
    });
    rx
}

/// Query GitHub for the latest published release version.
fn check_latest_release(current_version: &str) -> Result<Option<String>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let response = client
        .get(LATEST_RELEASE_API_URL)
        .header(reqwest::header::USER_AGENT, "tui-game")
        .send()?
        .error_for_status()?
        .json::<LatestReleaseResponse>()?;
    let latest_tag = normalized_tag(&response.tag_name);
    if is_remote_version_newer(current_version, &latest_tag) {
        Ok(Some(latest_tag))
    } else {
        Ok(None)
    }
}

/// Compare `vX.Y.Z` and `X.Y.Z` style version strings.
fn is_remote_version_newer(current_version: &str, remote_version: &str) -> bool {
    let current = parse_version_segments(current_version);
    let remote = parse_version_segments(remote_version);
    let max_len = current.len().max(remote.len());
    for idx in 0..max_len {
        let current_part = *current.get(idx).unwrap_or(&0);
        let remote_part = *remote.get(idx).unwrap_or(&0);
        if remote_part > current_part {
            return true;
        }
        if remote_part < current_part {
            return false;
        }
    }
    false
}

fn parse_version_segments(version: &str) -> Vec<u32> {
    let trimmed = version
        .trim()
        .trim_start_matches('v')
        .trim_start_matches('V');
    trimmed
        .split('.')
        .map(|part| part.parse::<u32>().unwrap_or(0))
        .collect()
}
