//! 运行阶段主事件循环

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Instant;

use crate::LuaRuntimeState;
use crate::host_engine::boot::preload::init_environment::{
    HostInputEvent, ResizeEvent, TerminalSize,
};
use crate::host_engine::boot::preload::lua_runtime::api::LuaEvent;
use crate::host_engine::boot::preload::lua_runtime::{HostLuaBridge, HostLuaMessage};
use crate::host_engine::boot::preload::overlay_modules::{OverlayPackage, OverlayRegistry};
use crate::host_engine::boot::preload::state_machine::HostStateMachine;
use crate::host_engine::constant::{ROOT_UI_MIN_HEIGHT, ROOT_UI_MIN_WIDTH};
use crate::host_engine::keybind::keybind_manager::KeybindManager;
use crate::host_engine::package::package_manager::PackageManager;
use crate::host_engine::runtime::frame_rate::FrameRateController;
use crate::host_engine::runtime::game_engine::best_score_store;
use crate::host_engine::runtime::overlay::{OverlaySession, OverlaySessionKind};
use crate::host_engine::runtime::renderer::RendererState;
use crate::host_engine::runtime::ui::pages::{
    GameListPage, HomePage, KeybindSystemPage, ModBossListPage, ModGameListPage, ModHubPage,
    ModScreensaverListPage, SettingDisplayPage, SettingKeybindPage, SettingLanguagePage,
    SettingMemoryPage, SettingPage, SettingSecurityPage, StorageDetailsPage, WarningClearCachePage,
    WarningClearDataPage, WarningModPage, WarningNeededSizePage, WarningSecurityPage,
};
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiManager, UiNavigation};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;
use crate::host_engine::runtime::ui_runtime::ActiveUiPage;
use crate::host_engine::runtime::ui_state::needed_size_state::{
    NeededSizeMode, NeededSizeRootState,
};
use crate::host_engine::storage::cache_store::CacheStore;
use crate::host_engine::storage::profile_store::ProfileStore;
use crate::host_engine::theme::ThemeManager;
use serde_json::Value;

type RuntimeLoopResult<T> = Result<T, Box<dyn std::error::Error>>;
const UI_EVENT_QUEUE_LIMIT: usize = 256;

/// 运行最小宿主事件循环。
///
/// 当前阶段先保持 runtime 持久化，后续会在这里接入 UI Lua 脚本渲染、状态机切换和
/// 存储更新。退出条件暂定为 Ctrl+C、Esc 或 Q。
pub(crate) fn run(
    input_receiver: &Receiver<HostInputEvent>,
    host_bridge: &HostLuaBridge,
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
    host_state_machine: &mut HostStateMachine,
    overlay_registry: &OverlayRegistry,
    profile_store: Arc<ProfileStore>,
    _cache_store: Arc<CacheStore>,
    package_manager: Arc<PackageManager>,
    keybind_manager: Arc<KeybindManager>,
    theme_manager: Arc<ThemeManager>,
) -> RuntimeLoopResult<()> {
    let mut renderer_state = RendererState::new();
    let initial_page_key = current_page_key(host_bridge, UiPageKey::Home);
    crate::host_engine::runtime::ui_runtime::ensure_page(
        lua_runtime,
        active_ui_page,
        initial_page_key,
    )?;
    let mut ui_manager = create_ui_manager(
        initial_page_key,
        host_bridge.runtime_context().terminal_size,
        profile_store,
        package_manager,
        keybind_manager,
        theme_manager,
    );
    ui_manager.set_action_hints(action_hints(active_ui_page.action_value()));
    render_active_page(&mut ui_manager, host_bridge, initial_page_key)?;
    crate::host_engine::runtime::renderer::render_canvas(host_bridge, &mut renderer_state)?;
    let mut event_queue = VecDeque::new();
    let mut last_tick_at = Instant::now();
    let mut frame_rate_controller =
        FrameRateController::root_ui(active_ui_page.root_idle_threshold());
    let mut was_running_game = false;
    let mut is_focused = true;
    let mut overlay_session: Option<OverlaySession> = None;
    loop {
        if overlay_session.is_none() {
            if !active_ui_page.has_game_session() {
                frame_rate_controller.set_root_idle_timeout(active_ui_page.root_idle_threshold());
            }
            if active_ui_page.has_game_session() && !was_running_game {
                if let Some(game_session) = active_ui_page.game_session() {
                    frame_rate_controller = FrameRateController::game(
                        game_session.afk_time_secs(),
                        game_session.target_fps(),
                    );
                }
                was_running_game = true;
            } else if !active_ui_page.has_game_session() && was_running_game {
                frame_rate_controller =
                    FrameRateController::root_ui(active_ui_page.root_idle_threshold());
                was_running_game = false;
            }
        }

        match input_receiver.recv_timeout(frame_rate_controller.frame_interval()) {
            Ok(HostInputEvent::ExitRequested) => break,
            Ok(HostInputEvent::FocusLost) => {
                is_focused = false;
                event_queue.clear();
                let mut ctx = host_bridge.runtime_context();
                ctx.is_focused = false;
                host_bridge.set_runtime_context(ctx);
                if overlay_session.is_none() {
                    enqueue_limited(&mut event_queue, LuaEvent::FocusLost);
                }
            }
            Ok(HostInputEvent::FocusGained) => {
                is_focused = true;
                let mut ctx = host_bridge.runtime_context();
                ctx.is_focused = true;
                host_bridge.set_runtime_context(ctx);
                if overlay_session.is_none() {
                    enqueue_limited(&mut event_queue, LuaEvent::FocusGained);
                }
            }
            Ok(HostInputEvent::Resize(resize_event)) => {
                if overlay_session.is_none() {
                    enqueue_limited(
                        &mut event_queue,
                        LuaEvent::Resize {
                            width: resize_event.width,
                            height: resize_event.height,
                        },
                    );
                }
                update_resize_surface(host_bridge, resize_event)?;
                renderer_state.request_full_redraw();
            }
            Ok(HostInputEvent::Key { key, status }) if is_focused => {
                frame_rate_controller.mark_input();
                if status == "press" {
                    let overlay_was_active = overlay_session.is_some();
                    if handle_global_key(
                        key.as_str(),
                        host_bridge,
                        active_ui_page,
                        host_state_machine,
                        overlay_registry,
                        &mut overlay_session,
                        &mut renderer_state,
                    )? {
                        if overlay_session.is_some() {
                            frame_rate_controller = FrameRateController::overlay();
                        } else if overlay_was_active {
                            frame_rate_controller = frame_rate_for_current_runtime(active_ui_page);
                            was_running_game = active_ui_page.has_game_session();
                        }
                        event_queue.clear();
                        continue;
                    }
                    if overlay_session.is_some() {
                        continue;
                    }
                    enqueue_key_events(
                        &mut event_queue,
                        host_bridge,
                        active_ui_page,
                        key.as_str(),
                        status.as_str(),
                    );
                } else {
                    if overlay_session.is_none() {
                        enqueue_limited(&mut event_queue, LuaEvent::Key { name: key, status });
                    }
                }
            }
            Ok(HostInputEvent::Key { .. }) => {
                // Drop key events when unfocused
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }

        let now = Instant::now();
        let tick_dt_ms = now.duration_since(last_tick_at).as_millis() as u64;
        last_tick_at = now;

        if let Some(overlay_session) = overlay_session.as_mut() {
            overlay_session.update_and_render()?;
            crate::host_engine::runtime::renderer::render_canvas(host_bridge, &mut renderer_state)?;
            continue;
        }

        if active_ui_page.has_game_session() {
            dispatch_game_event_queue(lua_runtime, active_ui_page, &mut event_queue)?;
            handle_game_event(
                lua_runtime,
                active_ui_page,
                LuaEvent::Tick { dt_ms: tick_dt_ms },
            )?;
            render_game(lua_runtime, active_ui_page)?;
            crate::host_engine::runtime::renderer::render_canvas(host_bridge, &mut renderer_state)?;
            handle_game_messages(
                lua_runtime,
                host_bridge,
                active_ui_page,
                host_state_machine,
                &mut event_queue,
            )?;
            continue;
        }

        if frame_rate_controller.is_root_idle() && active_ui_page.should_auto_enter_screensaver() {
            if let Some(uid) = active_ui_page.next_screensaver_overlay_uid() {
                let package = overlay_package_by_uid(
                    &active_ui_page.overlay_registry().screensavers,
                    uid.as_str(),
                );
                toggle_overlay(
                    host_bridge,
                    package,
                    OverlaySessionKind::Screensaver,
                    &mut overlay_session,
                    &mut renderer_state,
                )?;
                if overlay_session.is_some() {
                    frame_rate_controller = FrameRateController::overlay();
                    event_queue.clear();
                    continue;
                }
            }
        }

        let page_key = current_page_key(host_bridge, ui_manager.active_page());
        if active_ui_page.page_key() != page_key {
            renderer_state.request_full_redraw();
        }
        crate::host_engine::runtime::ui_runtime::ensure_page(
            lua_runtime,
            active_ui_page,
            page_key,
        )?;
        dispatch_event_queue(
            &mut ui_manager,
            lua_runtime,
            active_ui_page,
            host_state_machine,
            host_bridge,
            &mut event_queue,
            &mut renderer_state,
        )?;
        if active_ui_page.has_game_session() {
            handle_game_event(
                lua_runtime,
                active_ui_page,
                LuaEvent::Tick { dt_ms: tick_dt_ms },
            )?;
            render_game(lua_runtime, active_ui_page)?;
            crate::host_engine::runtime::renderer::render_canvas(host_bridge, &mut renderer_state)?;
            handle_game_messages(
                lua_runtime,
                host_bridge,
                active_ui_page,
                host_state_machine,
                &mut event_queue,
            )?;
            continue;
        }
        ui_manager.handle_event(&UiEvent::Tick { dt_ms: tick_dt_ms })?;
        handle_ui_navigation(
            &mut ui_manager,
            lua_runtime,
            active_ui_page,
            host_state_machine,
            host_bridge,
            &mut renderer_state,
        )?;
        let page_key = current_page_key(host_bridge, ui_manager.active_page());
        if active_ui_page.page_key() != page_key {
            renderer_state.request_full_redraw();
        }
        crate::host_engine::runtime::ui_runtime::ensure_page(
            lua_runtime,
            active_ui_page,
            page_key,
        )?;
        render_active_page(&mut ui_manager, host_bridge, page_key)?;
        crate::host_engine::runtime::renderer::render_canvas(host_bridge, &mut renderer_state)?;

        if should_exit(host_bridge) {
            break;
        }
    }

    Ok(())
}

fn handle_global_key(
    key: &str,
    host_bridge: &HostLuaBridge,
    active_ui_page: &mut ActiveUiPage,
    host_state_machine: &mut HostStateMachine,
    _overlay_registry: &OverlayRegistry,
    overlay_session: &mut Option<OverlaySession>,
    renderer_state: &mut RendererState,
) -> RuntimeLoopResult<bool> {
    match key.to_ascii_lowercase().as_str() {
        "f2" => {
            let package = active_ui_page
                .next_screensaver_overlay_uid()
                .and_then(|uid| {
                    overlay_package_by_uid(
                        &active_ui_page.overlay_registry().screensavers,
                        uid.as_str(),
                    )
                });
            toggle_overlay(
                host_bridge,
                package,
                OverlaySessionKind::Screensaver,
                overlay_session,
                renderer_state,
            )?;
            Ok(true)
        }
        "f3" => {
            let package = active_ui_page.next_boss_overlay_uid().and_then(|uid| {
                overlay_package_by_uid(&active_ui_page.overlay_registry().bosses, uid.as_str())
            });
            toggle_overlay(
                host_bridge,
                package,
                OverlaySessionKind::Boss,
                overlay_session,
                renderer_state,
            )?;
            Ok(true)
        }
        "f4" => {
            if active_ui_page.has_game_session() {
                active_ui_page.clear_game_session();
                host_state_machine.game_list_state =
                    crate::host_engine::boot::preload::state_machine::GameListState::List;
                renderer_state.request_full_redraw();
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn create_ui_manager(
    active_page: UiPageKey,
    terminal_size: TerminalSize,
    profile_store: Arc<ProfileStore>,
    package_manager: Arc<PackageManager>,
    keybind_manager: Arc<KeybindManager>,
    theme_manager: Arc<ThemeManager>,
) -> UiManager {
    let mut ui_manager = UiManager::new(
        active_page,
        UiContext {
            terminal_size,
            i18n: Arc::new(crate::host_engine::boot::i18n::text()),
            themes: theme_manager,
            keybinds: keybind_manager,
            packages: package_manager,
            profiles: profile_store,
            action_hints: Default::default(),
            mod_warning_package_name: String::new(),
            needed_size_mode: NeededSizeMode::Root,
        },
    );
    ui_manager.register_page(Box::new(HomePage::new()));
    ui_manager.register_page(Box::new(GameListPage::new()));
    ui_manager.register_page(Box::new(SettingPage::new()));
    ui_manager.register_page(Box::new(SettingDisplayPage::new()));
    ui_manager.register_page(Box::new(SettingKeybindPage::new()));
    ui_manager.register_page(Box::new(SettingLanguagePage::new()));
    ui_manager.register_page(Box::new(SettingMemoryPage::new()));
    ui_manager.register_page(Box::new(SettingSecurityPage::new()));
    ui_manager.register_page(Box::new(ModHubPage::new()));
    ui_manager.register_page(Box::new(KeybindSystemPage::new()));
    ui_manager.register_page(Box::new(ModGameListPage::new()));
    ui_manager.register_page(Box::new(ModScreensaverListPage::new()));
    ui_manager.register_page(Box::new(ModBossListPage::new()));
    ui_manager.register_page(Box::new(StorageDetailsPage::new()));
    ui_manager.register_page(Box::new(WarningSecurityPage::new()));
    ui_manager.register_page(Box::new(WarningModPage::new()));
    ui_manager.register_page(Box::new(WarningClearCachePage::new()));
    ui_manager.register_page(Box::new(WarningClearDataPage::new()));
    ui_manager.register_page(Box::new(WarningNeededSizePage::new()));
    ui_manager
}

fn handle_ui_navigation(
    ui_manager: &mut UiManager,
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
    host_state_machine: &mut HostStateMachine,
    host_bridge: &HostLuaBridge,
    renderer_state: &mut RendererState,
) -> RuntimeLoopResult<()> {
    let Some(navigation) = ui_manager.take_navigation() else {
        return Ok(());
    };
    match navigation {
        UiNavigation::Exit => {
            host_bridge.push_message(HostLuaMessage::ExitGame);
        }
        UiNavigation::Page(page_key) => {
            crate::host_engine::runtime::ui_runtime::ensure_page(
                lua_runtime,
                active_ui_page,
                page_key,
            )?;
            ui_manager.navigate_to(page_key)?;
            ui_manager.set_action_hints(action_hints(active_ui_page.action_value()));
            renderer_state.request_full_redraw();
        }
        UiNavigation::StartGame(game_uid) => {
            if active_ui_page.start_game(lua_runtime, game_uid.as_str())? {
                host_state_machine.game_list_state =
                    crate::host_engine::boot::preload::state_machine::GameListState::Game;
                renderer_state.request_full_redraw();
            }
        }
    }
    Ok(())
}

fn ui_event_from_lua_event(event: LuaEvent) -> UiEvent {
    match event {
        LuaEvent::Action { name, status } => UiEvent::Action { name, status },
        LuaEvent::Key { name, status } => UiEvent::Key { name, status },
        LuaEvent::Resize { width, height } => UiEvent::Resize { width, height },
        LuaEvent::Tick { dt_ms } => UiEvent::Tick { dt_ms },
        LuaEvent::FocusGained => UiEvent::FocusGained,
        LuaEvent::FocusLost => UiEvent::FocusLost,
    }
}

fn action_hints(actions: Value) -> std::collections::HashMap<String, String> {
    actions
        .as_object()
        .into_iter()
        .flat_map(|actions| actions.iter())
        .filter_map(|(name, value)| {
            let key = value.get("key")?;
            let key = match key {
                Value::String(key) => key.clone(),
                Value::Array(keys) => keys
                    .iter()
                    .filter_map(Value::as_str)
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                _ => String::new(),
            };
            (!key.is_empty()).then(|| (name.clone(), key))
        })
        .collect()
}

fn overlay_package_by_uid(packages: &[OverlayPackage], uid: &str) -> Option<OverlayPackage> {
    packages.iter().find(|package| package.uid == uid).cloned()
}

fn toggle_overlay(
    host_bridge: &HostLuaBridge,
    package: Option<crate::host_engine::boot::preload::overlay_modules::OverlayPackage>,
    target_kind: OverlaySessionKind,
    overlay_session: &mut Option<OverlaySession>,
    renderer_state: &mut RendererState,
) -> RuntimeLoopResult<()> {
    if overlay_session
        .as_ref()
        .map(|session| session.kind() == target_kind)
        .unwrap_or(false)
    {
        if let Some(session) = overlay_session.take() {
            session.stop(host_bridge);
        }
        renderer_state.request_full_redraw();
        return Ok(());
    }

    if let Some(session) = overlay_session.take() {
        session.stop(host_bridge);
    }

    if let Some(package) = package {
        *overlay_session = Some(OverlaySession::start(host_bridge, package)?);
        renderer_state.request_full_redraw();
    }
    Ok(())
}

fn frame_rate_for_current_runtime(active_ui_page: &ActiveUiPage) -> FrameRateController {
    if let Some(game_session) = active_ui_page.game_session() {
        FrameRateController::game(game_session.afk_time_secs(), game_session.target_fps())
    } else {
        FrameRateController::root_ui(active_ui_page.root_idle_threshold())
    }
}

fn current_page_key(host_bridge: &HostLuaBridge, active_page: UiPageKey) -> UiPageKey {
    let terminal_size = host_bridge.runtime_context().terminal_size;
    if terminal_size.width < ROOT_UI_MIN_WIDTH || terminal_size.height < ROOT_UI_MIN_HEIGHT {
        return UiPageKey::WarningNeededSize;
    }
    active_page
}

fn render_active_page(
    ui_manager: &mut UiManager,
    host_bridge: &HostLuaBridge,
    page_key: UiPageKey,
) -> RuntimeLoopResult<()> {
    if page_key == UiPageKey::WarningNeededSize {
        let needed_size_state = needed_size_root_state(host_bridge);
        ui_manager.set_needed_size_mode(needed_size_state.mode);
        ui_manager.navigate_to(UiPageKey::WarningNeededSize)?;
    }
    let mut canvas = Canvas::from_bridge(host_bridge.clone());
    ui_manager.render(&mut canvas)?;
    if page_key == UiPageKey::WarningNeededSize {
        return Ok(());
    }

    Ok(())
}

fn needed_size_root_state(host_bridge: &HostLuaBridge) -> NeededSizeRootState {
    NeededSizeRootState {
        actual: host_bridge.runtime_context().terminal_size,
        needed: TerminalSize {
            width: ROOT_UI_MIN_WIDTH,
            height: ROOT_UI_MIN_HEIGHT,
        },
        mode: NeededSizeMode::Root,
    }
}

fn enqueue_key_events(
    event_queue: &mut VecDeque<LuaEvent>,
    host_bridge: &HostLuaBridge,
    active_ui_page: &ActiveUiPage,
    key: &str,
    status: &str,
) {
    if let Some(game_session) = active_ui_page.game_session() {
        let runtime_context = host_bridge.runtime_context();
        if let Some(action) = game_session.action_for_key(&runtime_context.keybinds, key) {
            enqueue_limited(
                event_queue,
                LuaEvent::Action {
                    name: action,
                    status: status.to_string(),
                },
            );
            return;
        }
    }

    if let Some(action) = active_ui_page.action_for_key(key) {
        enqueue_limited(
            event_queue,
            LuaEvent::Action {
                name: action,
                status: status.to_string(),
            },
        );
        return;
    }

    enqueue_limited(
        event_queue,
        LuaEvent::Key {
            name: key.to_string(),
            status: status.to_string(),
        },
    );
}

fn dispatch_game_event_queue(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
    event_queue: &mut VecDeque<LuaEvent>,
) -> RuntimeLoopResult<()> {
    while let Some(event) = event_queue.pop_front() {
        handle_game_event(lua_runtime, active_ui_page, event)?;
    }
    Ok(())
}

fn handle_game_event(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
    event: LuaEvent,
) -> RuntimeLoopResult<()> {
    if let Some(game_session) = active_ui_page.game_session_mut() {
        game_session.handle_event(lua_runtime, event)?;
    }
    Ok(())
}

fn render_game(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &ActiveUiPage,
) -> RuntimeLoopResult<()> {
    if let Some(game_session) = active_ui_page.game_session() {
        game_session.render(lua_runtime)?;
    }
    Ok(())
}

fn handle_game_messages(
    lua_runtime: &LuaRuntimeState,
    host_bridge: &HostLuaBridge,
    active_ui_page: &mut ActiveUiPage,
    host_state_machine: &mut HostStateMachine,
    event_queue: &mut VecDeque<LuaEvent>,
) -> RuntimeLoopResult<()> {
    let mut should_exit_game = false;
    for message in host_bridge.drain_messages() {
        match message {
            HostLuaMessage::ExitGame => should_exit_game = true,
            HostLuaMessage::ClearEventQueue => event_queue.clear(),
            HostLuaMessage::SkipEventQueue => event_queue.clear(),
            HostLuaMessage::RenderNow => {}
            HostLuaMessage::SaveBestScore => {
                save_current_best_score(lua_runtime, host_bridge, active_ui_page)?;
            }
            HostLuaMessage::SaveGame => {
                // TODO: 接入持久化存储后在这里调用 save_game。
            }
        }
    }

    if should_exit_game {
        if let Some(game_session) = active_ui_page.game_session_mut() {
            game_session.exit_game(lua_runtime)?;
        }
        active_ui_page.clear_game_session();
        host_state_machine.game_list_state =
            crate::host_engine::boot::preload::state_machine::GameListState::List;
    }

    Ok(())
}

fn save_current_best_score(
    lua_runtime: &LuaRuntimeState,
    host_bridge: &HostLuaBridge,
    active_ui_page: &mut ActiveUiPage,
) -> RuntimeLoopResult<()> {
    let Some(game_session) = active_ui_page.game_session() else {
        return Ok(());
    };

    let best_string = game_session.save_best_score(lua_runtime)?;
    let best_scores = best_score_store::save_best_score(game_session.uid(), best_string.as_str())?;

    let mut runtime_context = host_bridge.runtime_context();
    runtime_context.best_scores = best_scores.clone();
    host_bridge.set_runtime_context(runtime_context);
    active_ui_page.refresh_best_scores(best_scores);

    Ok(())
}

fn enqueue_limited(event_queue: &mut VecDeque<LuaEvent>, event: LuaEvent) {
    if event_queue.len() >= UI_EVENT_QUEUE_LIMIT {
        let _ = event_queue.pop_front();
    }
    event_queue.push_back(event);
}

fn dispatch_event_queue(
    ui_manager: &mut UiManager,
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
    host_state_machine: &mut HostStateMachine,
    host_bridge: &HostLuaBridge,
    event_queue: &mut VecDeque<LuaEvent>,
    renderer_state: &mut RendererState,
) -> RuntimeLoopResult<()> {
    while let Some(event) = event_queue.pop_front() {
        ui_manager.handle_event(&ui_event_from_lua_event(event))?;
        handle_ui_navigation(
            ui_manager,
            lua_runtime,
            active_ui_page,
            host_state_machine,
            host_bridge,
            renderer_state,
        )?;
    }
    Ok(())
}

fn update_resize_surface(
    host_bridge: &HostLuaBridge,
    resize_event: ResizeEvent,
) -> RuntimeLoopResult<()> {
    let terminal_size = TerminalSize {
        width: resize_event.width,
        height: resize_event.height,
    };
    host_bridge.set_terminal_size(terminal_size);
    host_bridge.resize_canvas(terminal_size)?;
    Ok(())
}

fn should_exit(host_bridge: &HostLuaBridge) -> bool {
    host_bridge
        .drain_messages()
        .into_iter()
        .any(|message| matches!(message, HostLuaMessage::ExitGame))
}
