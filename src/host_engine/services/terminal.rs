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

pub struct TerminalService {
  guard: Option<TerminalGuard> // 终端守卫，支持终端开关
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
}

impl Drop for TerminalGuard {
  fn drop(&mut self) {
    // 恢复输入模式
    let _ = disable_raw_mode();

    // 恢复屏幕显示
    let stdout = &mut self._stdout;
    let _ = stdout.flush();
    let _ = execute!(stdout, Show, DisableMouseCapture, LeaveAlternateScreen);

    // 清理错误输出缓冲
    let _ = io::stderr().flush();
  }
}

impl TerminalService {
  pub fn new() -> Self {
    Self { guard: None }
  }

  pub fn enter(&mut self) {
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
        eprintln!("[Terminal] Failed to enter terminal mode: {}", error);
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