//! Alternate Screen 控制

use std::io::{self, Stdout};

use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

type AlternateScreenResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 进入备用屏幕
pub fn enter(stdout: &mut Stdout) -> AlternateScreenResult<()> {
    execute!(stdout, EnterAlternateScreen)?;
    Ok(())
}

/// 离开备用屏幕
pub fn leave() {
    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen);
}
