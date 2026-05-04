//! 运行阶段主事件循环

use std::collections::VecDeque;
use std::sync::mpsc::{RecvTimeoutError, Receiver};
use std::time::{Duration, Instant};

use crate::LuaRuntimeState;
use crate::host_engine::constant::{ROOT_UI_MIN_HEIGHT, ROOT_UI_MIN_WIDTH};
use crate::host_engine::boot::preload::init_environment::{
    HostInputEvent, ResizeEvent, TerminalSize,
};
use crate::host_engine::boot::preload::state_machine::HostStateMachine;
use crate::host_engine::boot::preload::lua_runtime::{HostLuaBridge, HostLuaMessage};
use crate::host_engine::boot::preload::lua_runtime::api::LuaEvent;
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;
use crate::host_engine::runtime::renderer::RendererState;
use crate::host_engine::runtime::ui_state::needed_size_state::{
    NeededSizeMode, NeededSizeRootState,
};
use crate::host_engine::runtime::ui_runtime::ActiveUiPage;

type RuntimeLoopResult<T> = Result<T, Box<dyn std::error::Error>>;
const UI_EVENT_QUEUE_LIMIT: usize = 256;
const UI_TICK_INTERVAL_MS: u64 = 16;

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
) -> RuntimeLoopResult<()> {
    let mut renderer_state = RendererState::new();
    let initial_page_key = current_page_key(host_bridge, host_state_machine);
    crate::host_engine::runtime::ui_runtime::ensure_page(
        lua_runtime,
        active_ui_page,
        initial_page_key,
    )?;
    render_active_page(lua_runtime, active_ui_page, host_bridge, initial_page_key)?;
    crate::host_engine::runtime::renderer::render_canvas(host_bridge, &mut renderer_state)?;
    let mut event_queue = VecDeque::new();
    let mut last_tick_at = Instant::now();
    loop {
        match input_receiver.recv_timeout(Duration::from_millis(UI_TICK_INTERVAL_MS)) {
            Ok(HostInputEvent::ExitRequested) => break,
            Ok(HostInputEvent::Resize(resize_event)) => {
                enqueue_limited(
                    &mut event_queue,
                    LuaEvent::Resize {
                        width: resize_event.width,
                        height: resize_event.height,
                    },
                );
                update_resize_surface(host_bridge, resize_event)?;
                renderer_state.request_full_redraw();
            }
            Ok(HostInputEvent::Key { key }) => {
                enqueue_key_events(&mut event_queue, active_ui_page, key.as_str());
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }

        let now = Instant::now();
        let tick_dt_ms = now.duration_since(last_tick_at).as_millis() as u64;
        last_tick_at = now;

        let page_key = current_page_key(host_bridge, host_state_machine);
        crate::host_engine::runtime::ui_runtime::ensure_page(
            lua_runtime,
            active_ui_page,
            page_key,
        )?;
        dispatch_event_queue(lua_runtime, active_ui_page, host_state_machine, &mut event_queue)?;
        crate::host_engine::runtime::ui_runtime::handle_event(
            lua_runtime,
            active_ui_page,
            host_state_machine,
            LuaEvent::Tick { dt_ms: tick_dt_ms },
        )?;
        let page_key = current_page_key(host_bridge, host_state_machine);
        crate::host_engine::runtime::ui_runtime::ensure_page(
            lua_runtime,
            active_ui_page,
            page_key,
        )?;
        render_active_page(lua_runtime, active_ui_page, host_bridge, page_key)?;
        crate::host_engine::runtime::renderer::render_canvas(host_bridge, &mut renderer_state)?;

        if should_exit(host_bridge) {
            break;
        }
    }

    Ok(())
}

fn current_page_key(
    host_bridge: &HostLuaBridge,
    host_state_machine: &HostStateMachine,
) -> UiPageKey {
    let terminal_size = host_bridge.runtime_context().terminal_size;
    if terminal_size.width < ROOT_UI_MIN_WIDTH || terminal_size.height < ROOT_UI_MIN_HEIGHT {
        return UiPageKey::WarningNeededSize;
    }
    UiPageKey::from_state_machine(host_state_machine)
}

fn render_active_page(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
    host_bridge: &HostLuaBridge,
    page_key: UiPageKey,
) -> RuntimeLoopResult<()> {
    if page_key == UiPageKey::WarningNeededSize {
        let needed_size_state = needed_size_root_state(host_bridge);
        // 尺寸提示保留原页面上下文，仅同步提示模式给 Lua 返回状态处理。
        // TODO: 游戏运行态接入后，按实际状态传入 NeededSizeMode::Game。
        active_ui_page.set_needed_size_mode(needed_size_state.mode);
        crate::host_engine::runtime::ui_runtime::render_needed_size(
            lua_runtime,
            needed_size_state,
        )?;
        return Ok(());
    }

    crate::host_engine::runtime::ui_runtime::render(lua_runtime, active_ui_page)?;
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
    active_ui_page: &ActiveUiPage,
    key: &str,
) {
    if let Some(action) = active_ui_page.action_for_key(key) {
        enqueue_limited(event_queue, LuaEvent::Action { name: action });
        return;
    }

    enqueue_limited(
        event_queue,
        LuaEvent::Key {
            name: key.to_string(),
        },
    );
}

fn enqueue_limited(event_queue: &mut VecDeque<LuaEvent>, event: LuaEvent) {
    if event_queue.len() >= UI_EVENT_QUEUE_LIMIT {
        let _ = event_queue.pop_front();
    }
    event_queue.push_back(event);
}

fn dispatch_event_queue(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
    host_state_machine: &mut HostStateMachine,
    event_queue: &mut VecDeque<LuaEvent>,
) -> RuntimeLoopResult<()> {
    while let Some(event) = event_queue.pop_front() {
        crate::host_engine::runtime::ui_runtime::handle_event(
            lua_runtime,
            active_ui_page,
            host_state_machine,
            event,
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
