//! 终端尺寸读取

type TerminalSizeResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 终端尺寸
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

/// 获取当前终端尺寸
pub fn current() -> TerminalSizeResult<TerminalSize> {
    let (width, height) = crossterm::terminal::size()?;
    Ok(TerminalSize { width, height })
}
