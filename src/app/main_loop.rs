// 应用主循环和顶层状态管理。包含 UI 状态机（AppState）、事件分发（handle_key_event）、主渲染循环（run）和各类辅助函数。是 main.rs 中拆分出的核心模块

use std::sync::mpsc::Receiver; // 接收版本检查结果
use std::thread; // 帧率控制睡眠
use std::time::{Duration, Instant}; // 帧预算和时间计算

use anyhow::Result; // 错误处理
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind}; // 终端事件（键盘、Resize）

use crate::app::content_cache; // 缓存查询（游戏列表）
use crate::app::continue_game; // 继续游戏同步
use crate::app::game_selection::{GameSelection, GameSelectionAction}; // 游戏选择页状态和动作
use crate::app::i18n; // 国际化文本
use crate::app::layout::{MENU_MIN_HEIGHT, MENU_MIN_WIDTH}; // 主菜单尺寸常量
use crate::app::menu::{Menu, MenuAction}; // 主菜单类型和渲染
use crate::app::placeholder_pages::{self, PlaceholderPage}; // 占位页面
use crate::app::settings; // 设置系统
use crate::core::runtime::{LaunchMode, launch_game}; // 游戏启动
use crate::core::save; // 存档操作
use crate::game::registry::GameDescriptor; // 游戏描述符
use crate::terminal::session::TerminalSession; // 终端会话
use crate::terminal::size_watcher; // 终端尺寸检测

pub const MAX_UI_EVENTS_PER_FRAME: usize = 256; // 每帧最多处理的 UI 事件数
pub const ACTIVE_FRAME_BUDGET: Duration = Duration::from_millis(16); // 活跃时的帧预算（~60fps）
pub const IDLE_FRAME_BUDGET: Duration = Duration::from_millis(250); // 空闲时的帧预算（~4fps）
pub const UI_IDLE_TIMEOUT: Duration = Duration::from_secs(60); // 进入空闲模式的超时

// 	顶层应用状态枚举
pub enum AppState {
    MainMenu { menu: Menu },
    GameSelection { ui: GameSelection },
    Settings { ui: settings::SettingsState },
    About,
    Continue,
    Exiting,
}

// 待确认的新游戏启动数据
pub struct PendingNewGameStart {
    pub target_game: GameDescriptor,
    pub saved_game_name: String,
}

// 根据当前状态返回最小终端尺寸
pub fn minimum_size_for_state(state: &AppState) -> (u16, u16) {
    match state {
        AppState::MainMenu { .. } => (MENU_MIN_WIDTH, MENU_MIN_HEIGHT),
        AppState::GameSelection { ui } => ui.minimum_size(),
        AppState::Settings { ui } => settings::minimum_size(ui),
        AppState::About | AppState::Continue => (MENU_MIN_WIDTH, MENU_MIN_HEIGHT),
        AppState::Exiting => (MENU_MIN_WIDTH, MENU_MIN_HEIGHT),
    }
}

// 判断是否需要持续刷新 UI（有对话框或按键捕获时返回 true）
pub fn should_keep_ui_animating(state: &AppState) -> bool {
    match state {
        AppState::Settings { ui } => {
            ui.mod_safe_dialog.is_some()
                || ui.cleanup_dialog.is_some()
                || ui.default_safe_mode_disable_dialog.is_some()
                || ui.security_success_at.is_some()
                || ui.keybind_capture.is_some()
        }
        _ => false,
    }
}

// 核心事件分发器：根据 AppState 分发按键到各子页面。包含主菜单导航、游戏选择确认、新游戏覆盖确认对话框、设置调度、占位页返回等
pub fn handle_key_event(
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
                                crate::utils::host_log::append_host_error(
                                    "host.error.clean_old_save_failed",
                                    &[("err", &format!("{err:#}"))],
                                );
                            }
                            if let Err(err) = launch_game(&pending.target_game, LaunchMode::New) {
                                crate::utils::host_log::append_host_error(
                                    "host.error.run_game_failed",
                                    &[
                                        ("game_id", pending.target_game.id.as_str()),
                                        ("err", &format!("{err:#}")),
                                    ],
                                );
                            }
                            crate::terminal::session::reset_terminal_after_runtime()?;
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
                            let saved_game_name =
                                continue_game::resolve_saved_game_name(&saved_game_id);
                            *pending_new_game_start = Some(PendingNewGameStart {
                                target_game: game,
                                saved_game_name,
                            });
                            return Ok(());
                        }
                        if let Err(err) = save::clear_active_game_save() {
                            crate::utils::host_log::append_host_error(
                                "host.error.clean_old_save_failed",
                                &[("err", &format!("{err:#}"))],
                            );
                        }
                        if let Err(err) = launch_game(&game, LaunchMode::New) {
                            crate::utils::host_log::append_host_error(
                                "host.error.run_game_failed",
                                &[("game_id", game.id.as_str()), ("err", &format!("{err:#}"))],
                            );
                        }
                        crate::terminal::session::reset_terminal_after_runtime()?;
                        *force_ui_full_redraw = true;
                        let games = content_cache::games();
                        ui.refresh_preserving_selection(games);
                    }
                }
            }
        }
        AppState::Settings { ui } => match settings::handle_key(ui, key) {
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

// 渲染新游戏覆盖确认模态框：全屏 Clear 背景，垂直居中显示警告消息和 Y/N 选项
pub fn render_new_game_confirm(frame: &mut ratatui::Frame<'_>, saved_game_name: &str) {
    use ratatui::layout::{Alignment, Constraint, Direction, Layout};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Clear, Paragraph, Wrap};

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

// 将主菜单动作转换为下一个 AppState：Play→游戏选择、Continue→启动继续游戏（失败仅记录日志）、Settings→设置页、About→占位页、Quit→退出
fn apply_menu_action(
    action: MenuAction,
    continue_game_id: Option<&str>,
    force_ui_full_redraw: &mut bool,
) -> AppState {
    match action {
        MenuAction::Play => AppState::GameSelection {
            ui: GameSelection::new(content_cache::games()),
        },
        MenuAction::Continue => {
            if let Some(game_id) = continue_game_id {
                let game = content_cache::games()
                    .into_iter()
                    .find(|g| g.id.eq_ignore_ascii_case(game_id));
                if let Some(game) = game {
                    if let Err(err) = launch_game(&game, LaunchMode::Continue) {
                        crate::utils::host_log::append_host_error(
                            "host.error.continue_game_failed",
                            &[("game_id", game.id.as_str()), ("err", &format!("{err:#}"))],
                        );
                    }
                    let _ = crate::terminal::session::reset_terminal_after_runtime();
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

// 主循环入口：初始化状态变量 → 循环执行帧预算控制、继续游戏同步、Mod 热重载轮询、版本更新检查、尺寸检测、事件收集与分发、页面渲染、帧率控制。接收到 Exiting 状态时退出循环
pub fn run(
    mut session: TerminalSession,
    runtime_version: String,
    update_check_rx: Receiver<Option<String>>,
) -> Result<()> {
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
            continue_game::sync_continue_item(menu);
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

        if update_hint.is_none()
            && let Ok(Some(latest_tag)) = update_check_rx.try_recv()
        {
            update_hint = Some(latest_tag);
            ui_dirty = true;
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
                        crate::app::menu::render_main_menu(
                            frame,
                            menu,
                            &runtime_version,
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
                            &runtime_version,
                            None,
                        );
                    }
                    AppState::Continue => {
                        placeholder_pages::render_placeholder(
                            frame,
                            PlaceholderPage::Continue,
                            &runtime_version,
                            None,
                        );
                    }
                    AppState::Exiting => {}
                })?;
                ui_dirty = false;
                last_size_ok = Some(true);
            }
        } else if should_draw {
            size_watcher::draw_size_warning(&size_state, min_width, min_height)?;
            ui_dirty = false;
            last_size_ok = Some(false);
        }

        let elapsed = frame_start.elapsed();
        if elapsed < frame_budget {
            thread::sleep(frame_budget - elapsed);
        }
    }

    drop(session);
    Ok(())
}
