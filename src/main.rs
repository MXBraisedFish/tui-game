use std::io::{self, Stdout};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use tui_game::app;
use tui_game::app::game_selection::{GameSelection, GameSelectionAction};
use tui_game::app::i18n;
use tui_game::app::layout::{MENU_MIN_HEIGHT, MENU_MIN_WIDTH};
use tui_game::app::menu::{Menu, MenuAction};
use tui_game::app::placeholder_pages::{self, PlaceholderPage};
use tui_game::app::settings;
use tui_game::lua_bridge::api::{
    LaunchMode, clear_active_game_save, latest_saved_game_id, run_game_script,
    take_terminal_dirty_from_lua,
};
use tui_game::lua_bridge::script_loader::{GameMeta, scan_scripts};
use tui_game::terminal::size_watcher;
use tui_game::updater::github::{
    CURRENT_VERSION_TAG, UpdateNotification, Updater, UpdaterEvent, run_update_binary,
};
use tui_game::utils::path_utils;

/// 应用的全局界面状态。
///
/// 主循环会根据该枚举决定当前渲染哪个页面，
/// 并把键盘事件分发给对应模块处理。
pub enum AppState {
    /// 主菜单界面。
    MainMenu { menu: Menu },
    /// 游戏选择界面。
    GameSelection { ui: GameSelection },
    /// 设置界面。
    Settings { ui: settings::SettingsState },
    /// 关于界面。
    About,
    /// 继续游戏流程占位状态。
    Continue,
    /// 程序准备退出。
    Exiting,
}

/// 新开游戏前的存档覆盖确认状态。
struct PendingNewGameStart {
    /// 用户准备启动的新游戏。
    target_game: GameMeta,
    /// 当前已有存档所属游戏名，用于确认提示。
    saved_game_name: String,
}

/// 终端生命周期封装。
///
/// 负责进入原始模式、切换备用屏、隐藏光标，
/// 并在离开作用域时尽量恢复终端状态。
struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    /// 初始化终端会话并返回可供 ratatui 使用的终端实例。
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
    /// 析构时恢复终端状态，作为整个应用的兜底清理逻辑。
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), Show, LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

/// 安装 panic hook，确保异常时也能恢复终端状态。
fn install_panic_hook() {
    let old = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let mut out = io::stdout();
        let _ = execute!(out, Show, LeaveAlternateScreen);
        old(panic_info);
    }));
}

/// 程序入口。
fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err:#}");
    }
}

/// 主程序核心入口。
///
/// 负责初始化终端、国际化、更新检查器与全局状态，
/// 然后驱动整个应用主循环。
fn run() -> Result<()> {
    // 安装 panic hook，避免异常时终端残留在 raw mode。
    install_panic_hook();
    // 初始化国际化系统。
    i18n::init("us-en")?;

    // 初始化终端会话。
    let mut session = TerminalSession::new()?;
    // 启动后台更新检查线程。
    let updater = Updater::spawn(CURRENT_VERSION_TAG);

    // 初始化全局状态和运行时变量。
    let mut update_notification: Option<UpdateNotification> = None;
    // 记录远端最新 release 版本。
    let mut latest_release_version = normalized_tag(CURRENT_VERSION_TAG);
    // 记录当前运行版本。
    let runtime_version = normalized_tag(CURRENT_VERSION_TAG);
    // 初始页面为主菜单。
    let mut state = AppState::MainMenu { menu: Menu::new() };
    let mut pending_new_game_start: Option<PendingNewGameStart> = None;
    // 标记是否需要在退出后执行 remove 字节码程序。
    let mut should_run_uninstall = false;

    let frame_budget = Duration::from_millis(16);

    // 主循环：处理后台事件、键盘输入、渲染和帧率控制。
    loop {
        let frame_start = Instant::now();

        // 非阻塞拉取后台更新检查结果。
        while let Some(event) = updater.try_recv() {
            match event {
                UpdaterEvent::LatestVersion(latest) => {
                    latest_release_version = latest.latest_version;
                }
                UpdaterEvent::NewVersion(notification) => {
                    update_notification = Some(notification);
                }
                UpdaterEvent::NoUpdate => {}
            }
        }

        // 主菜单下同步“继续游戏”的可用状态与目标游戏名。
        if let AppState::MainMenu { menu } = &mut state {
            sync_continue_item(menu);
        }

        // 处理键盘事件。
        if event::poll(Duration::from_millis(0))? {
            let ev = event::read()?;
            if let Event::Key(key) = ev {
                handle_key_event(
                    &mut state,
                    &mut pending_new_game_start,
                    &mut should_run_uninstall,
                    key,
                    update_notification.as_ref(),
                )?;
            }
        }

        // 如果 Lua 直接写过终端，则让 ratatui 清空自己的缓冲区。
        if take_terminal_dirty_from_lua() {
            session.terminal.clear()?;
        }

        if matches!(state, AppState::Exiting) {
            break;
        }

        // 根据当前页面动态计算最小终端尺寸。
        let (min_width, min_height) = minimum_size_for_state(&state);
        let size_state = size_watcher::check_size(min_width, min_height)?;

        if size_state.size_ok {
            session.terminal.draw(|frame| match &mut state {
                AppState::MainMenu { menu } => {
                    let version_hint = update_notification
                        .as_ref()
                        .map(|update| update.latest_version.as_str());
                    app::menu::render_main_menu(frame, menu, CURRENT_VERSION_TAG, version_hint);
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
                        Some(latest_release_version.as_str()),
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
        } else {
            // 尺寸不足时绘制警告覆盖层。
            size_watcher::draw_size_warning(&size_state, min_width, min_height)?;
        }

        // 帧率控制，降低空转 CPU 占用。
        let elapsed = frame_start.elapsed();
        if elapsed < frame_budget {
            thread::sleep(frame_budget - elapsed);
        }
    }

    // 终端恢复后再执行 remove 字节码程序，避免子进程继承异常终端状态。
    drop(session);
    if should_run_uninstall {
        let _ = run_remove_binary();
    }

    Ok(())
}

/// 根据当前页面状态返回所需的最小终端尺寸。
fn minimum_size_for_state(state: &AppState) -> (u16, u16) {
    match state {
        AppState::MainMenu { .. } => (MENU_MIN_WIDTH, MENU_MIN_HEIGHT),
        AppState::GameSelection { ui } => ui.minimum_size(),
        AppState::Settings { ui } => settings::minimum_size(ui),
        AppState::About | AppState::Continue => (MENU_MIN_WIDTH, MENU_MIN_HEIGHT),
        AppState::Exiting => (MENU_MIN_WIDTH, MENU_MIN_HEIGHT),
    }
}

/// 全局按键分发中心。
///
/// 先处理与页面无关的全局快捷键，再根据 `AppState`
/// 将事件转发给对应页面逻辑。
fn handle_key_event(
    state: &mut AppState,
    pending_new_game_start: &mut Option<PendingNewGameStart>,
    should_run_uninstall: &mut bool,
    key: KeyEvent,
    update_notification: Option<&UpdateNotification>,
) -> Result<()> {
    // 只响应按下事件，忽略重复输入和释放事件。
    if !matches!(key.kind, KeyEventKind::Press) {
        return Ok(());
    }

    // 全局更新快捷键，拉起 updata 字节码程序处理更新。
    if matches!(key.code, KeyCode::Char('u') | KeyCode::Char('U')) {
        if let Some(notification) = update_notification {
            if run_update_binary(notification).unwrap_or(false) {
                *state = AppState::Exiting;
                return Ok(());
            }
        }
    }

    // 只要离开游戏选择页，就清理“覆盖存档确认”状态。
    if !matches!(state, AppState::GameSelection { .. }) {
        *pending_new_game_start = None;
    }

    // 根据当前页面处理按键。
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
                    *state = apply_menu_action(action, menu.continue_game_id());
                }
            }
            _ => {}
        },
        AppState::GameSelection { ui } => {
            // 处理新游戏覆盖存档的确认流程。
            if pending_new_game_start.is_some() {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        let pending = pending_new_game_start.take();
                        if let Some(pending) = pending {
                            if let Err(err) = clear_active_game_save() {
                                eprintln!("Failed to clear active save slot: {err:#}");
                            }
                            if let Err(err) =
                                run_game_script(&pending.target_game.script_path, LaunchMode::New)
                            {
                                eprintln!(
                                    "Failed to run game '{}': {err:#}",
                                    pending.target_game.id
                                );
                            }
                            let games = scan_scripts().unwrap_or_default();
                            *ui = GameSelection::new(games);
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

            // 处理游戏选择页本体的高层动作。
            if let Some(action) = ui.handle_event(key) {
                match action {
                    GameSelectionAction::BackToMenu => {
                        *pending_new_game_start = None;
                        *state = AppState::MainMenu { menu: Menu::new() };
                    }
                    GameSelectionAction::LaunchGame(game) => {
                        if let Some(saved_game_id) = latest_saved_game_id() {
                            let saved_game_name =
                                i18n::t_or(&format!("game.{}.name", saved_game_id), &saved_game_id);
                            *pending_new_game_start = Some(PendingNewGameStart {
                                target_game: game,
                                saved_game_name,
                            });
                            return Ok(());
                        }
                        if let Err(err) = clear_active_game_save() {
                            eprintln!("Failed to clear active save slot: {err:#}");
                        }
                        if let Err(err) = run_game_script(&game.script_path, LaunchMode::New) {
                            eprintln!("Failed to run game '{}': {err:#}", game.id);
                        }
                        let games = scan_scripts().unwrap_or_default();
                        *ui = GameSelection::new(games);
                    }
                }
            }
        }

        AppState::Settings { ui } => {
            match settings::handle_key(ui, key.code) {
                settings::SettingsAction::None => {}
                settings::SettingsAction::BackToMenu => {
                    *state = AppState::MainMenu { menu: Menu::new() };
                }
                settings::SettingsAction::RunUninstall => {
                    if has_remove_binary().unwrap_or(false) {
                        *should_run_uninstall = true;
                        *state = AppState::Exiting;
                    }
                }
            }
        }

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

/// 渲染“已有存档，是否覆盖开始新游戏”的全屏确认提示。
fn render_new_game_confirm(frame: &mut ratatui::Frame<'_>, saved_game_name: &str) {
    use ratatui::layout::{Alignment, Constraint, Direction, Layout};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Clear, Paragraph, Wrap};

    // 使用 Clear 作为全屏覆盖层背景。
    let area = frame.area();
    frame.render_widget(Clear, area);

    let template = i18n::t("confirm.new_game_overwrite");
    // 按模板填入当前存档所属游戏名。
    let msg = if template.contains("{game}") {
        template.replace("{game}", saved_game_name)
    } else {
        format!("{template} {saved_game_name}")
    };

    let yes = i18n::t("confirm.new_game_yes");
    let no = i18n::t("confirm.new_game_no");

    // 将提示文本整体垂直居中。
    let center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(4), Constraint::Min(0)])
        .split(area);

    let p = Paragraph::new(vec![
        Line::from(Span::styled(
            msg,
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
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

/// 将主菜单动作转换为新的应用状态。
fn apply_menu_action(action: MenuAction, continue_game_id: Option<&str>) -> AppState {
    match action {
        MenuAction::Play => {
            let games = match scan_scripts() {
                Ok(found) => found,
                Err(_) => Vec::new(),
            };
            AppState::GameSelection {
                ui: GameSelection::new(games),
            }
        }

        // 继续游戏会尝试直接载入共享存档，然后返回游戏列表。
        MenuAction::Continue => {
            if let Some(game_id) = continue_game_id {
                let game = scan_scripts()
                    .unwrap_or_default()
                    .into_iter()
                    .find(|g| g.id.eq_ignore_ascii_case(game_id));
                if let Some(game) = game {
                    if let Err(err) = run_game_script(&game.script_path, LaunchMode::Continue) {
                        eprintln!("Failed to continue game '{}': {err:#}", game.id);
                    }
                }
            }
            let games = scan_scripts().unwrap_or_default();
            AppState::GameSelection {
                ui: GameSelection::new(games),
            }
        }

        MenuAction::Settings => AppState::Settings {
            ui: settings::SettingsState::new(),
        },

        MenuAction::About => AppState::About,

        MenuAction::Quit => AppState::Exiting,
    }
}

/// 执行当前目录下的 remove 字节码程序。
fn run_remove_binary() -> Result<bool> {
    let remove_bin = path_utils::remove_binary_file()?;
    if !remove_bin.exists() {
        return Ok(false);
    }

    let _child = Command::new(remove_bin).spawn()?;
    Ok(true)
}

/// 判断当前运行目录中是否存在可执行的 remove 字节码程序。
fn has_remove_binary() -> Result<bool> {
    Ok(path_utils::remove_binary_file()?.exists())
}

/// 根据共享存档槽同步主菜单中“继续游戏”的状态。
fn sync_continue_item(menu: &mut Menu) {
    let game_id = latest_saved_game_id();
    let game_name = game_id
        .as_deref()
        .map(|id| i18n::t_or(&format!("game.{}.name", id), id));
    menu.set_continue_target(game_id, game_name);
}

/// 将版本标签规范化为统一的 `vX.Y.Z` 形式。
fn normalized_tag(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with('v') || trimmed.starts_with('V') {
        format!("v{}", trimmed[1..].trim())
    } else {
        format!("v{}", trimmed)
    }
}



