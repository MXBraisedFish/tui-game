// 管理终端会话的完整生命周期。构造时进入 raw mode 和 alternate screen（TUI 模式），Drop 时自动恢复终端设置。还提供游戏结束后的终端重置功能

use std::io::{self, Stdout}; // 标准输出句柄

use anyhow::Result; // 错误处理
use crossterm::cursor::{Hide, Show}; // 隐藏/显示光标
use crossterm::execute; // 执行 ANSI 指令
use crossterm::terminal::{ // raw mode、alternate screen、清屏等终端控制
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

// ratatui 终端后端
use ratatui::Terminal; 
use ratatui::backend::CrosstermBackend;

use crate::terminal::renderer; // 画布缓存失效功能

// 终端会话管理器，封装 ratatui 的 Terminal<CrosstermBackend<Stdout>>
pub struct TerminalSession {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

// 构造器：开启 raw mode → 切换 alternate screen → 隐藏光标 → 创建 ratatui Terminal
impl TerminalSession {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut out = io::stdout();
        execute!(out, EnterAlternateScreen, Hide)?;
        let backend = CrosstermBackend::new(out);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

// Drop 实现：关闭 raw mode → 显示光标 → 退出 alternate screen，确保终端恢复
impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), Show, LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

// 构造器：开启 raw mode → 切换 alternate screen → 隐藏光标 → 创建 ratatui Terminal
pub fn reset_terminal_after_runtime() -> Result<()> {
    renderer::invalidate_canvas_cache();
    let mut out = io::stdout();
    execute!(out, Clear(ClearType::All), crossterm::cursor::MoveTo(0, 0))?;
    Ok(())
}
