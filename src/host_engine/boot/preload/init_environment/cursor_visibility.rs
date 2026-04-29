//! 光标显示状态控制

use std::io::{self, Stdout};

use crossterm::cursor::{Hide, Show};
use crossterm::execute;

type CursorVisibilityResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 隐藏光标
pub fn hide(stdout: &mut Stdout) -> CursorVisibilityResult<()> {
    execute!(stdout, Hide)?;
    Ok(())
}

/// 显示光标
pub fn show() {
    let mut stdout = io::stdout();
    let _ = execute!(stdout, Show);
}
