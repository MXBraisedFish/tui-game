// 官方标准输入输出
use std::io::{self, stdout, Stdout, Write};

// 光标控制
use crossterm::cursor::{Hide, Show};
// 鼠标事件控制
use crossterm::event::DisableMouseCapture;
// 终端命令执行
use crossterm::execute;
// 终端模式控制
use crossterm::terminal::{
  disable_raw_mode,
  enable_raw_mode,
  EnterAlternateScreen,
  LeaveAlternateScreen
};
use super::terminal_capabilities::TerminalCapabilities;

// 临时的日志函数
use super::LogService;

pub struct TerminalService {
  guard: Option<TerminalGuard>, // 终端守卫，支持终端开关
  capabilities: TerminalCapabilities // 终端能力
}

struct TerminalGuard {
  _stdout: Stdout // 保证持有所有权
}

impl TerminalGuard {
  fn enter() -> io::Result<Self> {
    // 获取标准输出
    let mut stdout = stdout();

    // 启用原始模式
    enable_raw_mode()?;
    // 切换屏幕，隐藏光标
    execute!(stdout, EnterAlternateScreen, Hide)?;

    // 返回守卫
    Ok(Self {_stdout: stdout})
  }

  pub fn force_restore() {
    // 禁用原始模式
    let _ = disable_raw_mode();

    // 获取标准输句柄
    let mut stdout = stdout();
    // 恢复终端
    let _ = execute!(stdout, Show, DisableMouseCapture, LeaveAlternateScreen);
    // 刷新缓冲区
    let _ = stdout.flush();
    // 刷新错误输出缓冲区
    let _ = io::stderr().flush();
  }
}

impl Drop for TerminalGuard {
  fn drop(&mut self) {
    // 恢复输入模式
    let _ = disable_raw_mode();

    // 恢复屏幕显示
    let stdout = &mut self._stdout;
    let _ = execute!(stdout, Show, DisableMouseCapture, LeaveAlternateScreen);
    let _ = stdout.flush();

    // 清理错误输出缓冲
    let _ = io::stderr().flush();
  }
}

impl TerminalService {
  pub fn new() -> Self {
    Self { 
      guard: None,
      capabilities: TerminalCapabilities::detected()
    }
  }

  // 终端能力获取函数
  pub fn capabilities(&self) -> &TerminalCapabilities {
    &self.capabilities
  }

  pub fn enter(&mut self, services: &mut LogService) {
    //防止重复进入
    if self.guard.is_some() {
      return;
    }

    // 尝试进入终端模式
    match TerminalGuard::enter() {
      Ok(guard) => {
        // 创建守卫
        self.guard = Some(guard);
      }
      Err(error) => {
        // TODO: 这里的警告应该国际化或者写入日志而不是直接打印
        services.error(format!("[Terminal] Failed to enter terminal mode: {}", error));
      }
    }
  }

  // 退出，丢弃守卫，触发drop
  pub fn exit(&mut self) {
    self.guard = None;
  }

  // 状态检查
  pub fn is_active(&self) -> bool {
    self.guard.is_some()
  }
}