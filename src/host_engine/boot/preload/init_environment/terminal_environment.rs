//! 终端环境生命周期管理

use std::io::{self, Stdout};

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use super::{alternate_screen, cursor_visibility, raw_mode};

type TerminalEnvironmentResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 已接管的终端环境
pub struct TerminalEnvironment {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalEnvironment {
    /// 启用 Raw Mode、进入备用屏幕、隐藏光标并创建 Ratatui 终端
    pub fn enter() -> TerminalEnvironmentResult<Self> {
        raw_mode::enable()?;

        let mut stdout = io::stdout();
        alternate_screen::enter(&mut stdout)?;
        cursor_visibility::hide(&mut stdout)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalEnvironment {
    fn drop(&mut self) {
        raw_mode::disable();
        cursor_visibility::show();
        alternate_screen::leave();
        let _ = self.terminal.show_cursor();
    }
}
