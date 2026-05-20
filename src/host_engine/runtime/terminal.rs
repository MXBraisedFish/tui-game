//! 运行阶段终端接管

use std::io::Write;

use crossterm::cursor::Show;
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};

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

/// 尽可能恢复终端状态。
///
/// 该函数用于 panic hook 和异常退出兜底路径，内部不得继续向外传播错误。
pub fn force_restore() {
    let _ = disable_raw_mode();

    let mut stdout = std::io::stdout();
    let _ = execute!(
        stdout,
        Show,
        DisableMouseCapture,
        LeaveAlternateScreen
    );
    let _ = stdout.flush();
    let _ = std::io::stderr().flush();
}

impl RuntimeTerminalSession {
    /// 暂时暴露终端会话是否仍被持有，避免运行循环占位阶段误删字段。
    pub fn is_active(&self) -> bool {
        let _ = &self.terminal_environment;
        true
    }
}
