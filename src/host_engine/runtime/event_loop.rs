//! 运行阶段主事件循环

use std::sync::mpsc::Receiver;

use crate::host_engine::boot::preload::init_environment::HostInputEvent;

type RuntimeLoopResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 运行最小宿主事件循环。
///
/// 当前阶段先保持 runtime 持久化，后续会在这里接入 UI Lua 脚本渲染、状态机切换和
/// 存储更新。退出条件暂定为 Ctrl+C、Esc 或 Q。
pub fn run(input_receiver: &Receiver<HostInputEvent>) -> RuntimeLoopResult<()> {
    loop {
        match input_receiver.recv()? {
            HostInputEvent::ExitRequested => break,
            HostInputEvent::Key { key } if is_exit_key(key.as_str()) => break,
            HostInputEvent::Key { .. } | HostInputEvent::Resize(_) => {}
        }
    }

    Ok(())
}

fn is_exit_key(key: &str) -> bool {
    matches!(key, "esc" | "q" | "Q")
}
