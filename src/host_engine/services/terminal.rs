// 官方标准输入输出
use std::io::{self, stdout, Stdout, Write};

// 光标控制
use crossterm::cursor::{Hide, Show};
// 鼠标和焦点事件控制
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
// 终端命令执行
use crossterm::execute;
// 终端模式控制
use super::terminal_capabilities::TerminalCapabilities;
use crossterm::terminal::{
  disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

// 日志
use super::{LogService, LogSource};

pub struct TerminalService {
  surface: Option<TerminalSurface>,   // 终端守卫，支持终端开关
  capabilities: TerminalCapabilities, // 终端能力
}

// 终端表面活动结构体
struct TerminalSurface {
  stdout: Stdout, // 终端输出流
  active: bool,   // 恢复是否仍然需要运行
}

impl TerminalSurface {
  fn enter() -> io::Result<Self> {
    // 启用原始模式
    enable_raw_mode()?;

    // 获取标准输出
    let mut stdout = stdout();

    // 进入屏幕 → 启用输入报告 → 隐藏光标 → 刷新
    execute!(stdout, EnterAlternateScreen)?;
    execute!(stdout, EnableMouseCapture)?;
    execute!(stdout, EnableFocusChange)?;
    execute!(stdout, Hide)?;
    stdout.flush()?;

    // 返回守卫
    Ok(Self {
      stdout,
      active: true,
    })
  }

  // 写入访问
  fn writer(&mut self) -> &mut Stdout {
    &mut self.stdout
  }

  fn restore(&mut self) {
    if !self.active {
      return;
    }

    // 恢复终端
    let _ = execute!(self.stdout, Show);
    let _ = execute!(self.stdout, DisableFocusChange);
    let _ = execute!(self.stdout, DisableMouseCapture);
    let _ = execute!(self.stdout, LeaveAlternateScreen);
    let _ = self.stdout.flush();

    // 关闭原始模式
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

  // 终端能力获取函数
  pub fn capabilities(&self) -> &TerminalCapabilities {
    &self.capabilities
  }

  pub fn enter(&mut self, services: &mut LogService) {
    //防止重复进入
    if self.surface.is_some() {
      return;
    }

    // 尝试进入终端模式
    match TerminalSurface::enter() {
      Ok(surface) => {
        // 创建守卫
        self.surface = Some(surface);
      }
      Err(error) => {
        // TODO: 这里的警告应该国际化或者写入日志而不是直接打印
        services.error(
          LogSource::Storage,
          format!("[Terminal] Failed to enter terminal mode: {}", error),
        );
      }
    }
  }

  // 退出，丢弃守卫，触发drop
  pub fn exit(&mut self) {
    self.surface = None;
  }

  // 状态检查
  pub fn is_active(&self) -> bool {
    self.surface.is_some()
  }

  // 写访问
  pub fn writer_mut(&mut self) -> Option<&mut Stdout> {
    self.surface.as_mut().map(|surface| surface.writer())
  }

  /// 清屏并归位光标。
  ///
  /// 图片变化时由 runtime 调用，确保旧图片残留被清除后再重绘字符层。
  pub fn clear_all_and_home(&mut self) -> io::Result<()> {
    use crossterm::terminal::{Clear, ClearType};
    use crossterm::QueueableCommand;
    use crossterm::cursor::MoveTo;

    if let Some(stdout) = self.writer_mut() {
      stdout.queue(Clear(ClearType::All))?;
      stdout.queue(MoveTo(0, 0))?;
      stdout.flush()?;
    }

    Ok(())
  }

  // 紧急恢复
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
