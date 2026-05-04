//! 运行阶段终端接管

use crate::host_engine::boot::preload::init_environment::terminal_environment::TerminalEnvironment;

type RuntimeTerminalResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 运行期终端会话。
///
/// 加载阶段使用普通终端输出；进入运行循环前才切换 raw mode、备用屏幕和隐藏光标，
/// 避免加载条被备用屏幕切走后恢复普通屏幕时残留旧进度。
pub struct RuntimeTerminalSession {
    terminal_environment: TerminalEnvironment,
}

/// 接管终端，进入 TUI 运行环境。
pub fn enter() -> RuntimeTerminalResult<RuntimeTerminalSession> {
    Ok(RuntimeTerminalSession {
        terminal_environment: TerminalEnvironment::enter()?,
    })
}

impl RuntimeTerminalSession {
    /// 暂时暴露终端会话是否仍被持有，避免运行循环占位阶段误删字段。
    pub fn is_active(&self) -> bool {
        let _ = &self.terminal_environment;
        true
    }
}
