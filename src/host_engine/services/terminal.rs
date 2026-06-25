use std::io::{self, Stdout, Write, stdout};

use crossterm::cursor::{Hide, Show};

use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};

use crossterm::execute;

use super::terminal_capabilities::TerminalCapabilities;
use crossterm::terminal::{
  EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use super::{LogService, LogSource};

/// 终端服务，管理原始模式和交替屏幕的进入与退出
pub struct TerminalService {
  surface: Option<TerminalSurface>,
  capabilities: TerminalCapabilities,
}

struct TerminalSurface {
  stdout: Stdout,
  active: bool,
}

impl TerminalSurface {
  // 初始化终端原始模式：启用 raw mode、交替屏幕、鼠标捕获、焦点事件，隐藏光标
  fn enter() -> io::Result<Self> {
    enable_raw_mode()?;

    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen)?;
    execute!(stdout, EnableMouseCapture)?;
    execute!(stdout, EnableFocusChange)?;
    execute!(stdout, Hide)?;
    stdout.flush()?;

    Ok(Self {
      stdout,
      active: true,
    })
  }

  fn writer(&mut self) -> &mut Stdout {
    &mut self.stdout
  }

  // 恢复终端到正常模式：显示光标、禁用交替屏幕、鼠标捕获和 raw mode
  fn restore(&mut self) {
    if !self.active {
      return;
    }

    let _ = execute!(self.stdout, Show);
    let _ = execute!(self.stdout, DisableFocusChange);
    let _ = execute!(self.stdout, DisableMouseCapture);
    let _ = execute!(self.stdout, LeaveAlternateScreen);
    let _ = self.stdout.flush();

    let _ = disable_raw_mode();
    let _ = io::stderr().flush();

    self.active = false;
  }
}

impl Drop for TerminalSurface {
  fn drop(&mut self) {
    self.restore();
  }
}

impl TerminalService {
  pub fn new() -> Self {
    Self {
      surface: None,
      capabilities: TerminalCapabilities::detect(),
    }
  }

  pub fn capabilities(&self) -> &TerminalCapabilities {
    &self.capabilities
  }

  /// 进入终端原始模式（启用交替屏幕、鼠标捕获和焦点事件）
  pub fn enter(&mut self, services: &mut LogService) {
    if self.surface.is_some() {
      return;
    }

    match TerminalSurface::enter() {
      Ok(surface) => {
        self.surface = Some(surface);
      }
      Err(error) => {
        services.error(
          LogSource::Storage,
          format!("[Terminal] Failed to enter terminal mode: {}", error),
        );
      }
    }
  }

  /// 退出终端原始模式
  pub fn exit(&mut self) {
    self.surface = None;
  }

  pub fn is_active(&self) -> bool {
    self.surface.is_some()
  }

  pub fn writer_mut(&mut self) -> Option<&mut Stdout> {
    self.surface.as_mut().map(|surface| surface.writer())
  }

  /// 清屏并将光标归位到 (0, 0)
  pub fn clear_all_and_home(&mut self) -> io::Result<()> {
    use crossterm::QueueableCommand;
    use crossterm::cursor::MoveTo;
    use crossterm::terminal::{Clear, ClearType};

    if let Some(stdout) = self.writer_mut() {
      stdout.queue(Clear(ClearType::All))?;
      stdout.queue(MoveTo(0, 0))?;
      stdout.flush()?;
    }

    Ok(())
  }

  /// 强制恢复终端设置（用于异常退出时的清理）
  pub fn force_restore() {
    let _ = disable_raw_mode();

    let mut stdout = stdout();

    let _ = execute!(stdout, Show);
    let _ = execute!(stdout, DisableFocusChange);
    let _ = execute!(stdout, DisableMouseCapture);
    let _ = execute!(stdout, LeaveAlternateScreen);
    let _ = stdout.flush();

    let _ = io::stderr().flush();
  }
}
