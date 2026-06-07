use std::io::{self, Write};

use crossterm::{
  QueueableCommand,
  cursor::MoveTo,
  style::Print,
  terminal::{Clear, ClearType},
};

use super::{buffer::CanvasBuffer, cell::CanvasCell};
use crate::host_engine::services::TerminalService;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasService {
  buffer: CanvasBuffer,
}

impl CanvasService {
  pub fn new() -> Self {
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
    Self {
      buffer: CanvasBuffer::new(width, height),
    }
  }

  pub fn begin_frame(&mut self) {
    // 为运行生命周期保留。当前最小画布没有帧局部状态。
  }

  // 清理画布
  pub fn clear(&mut self) {
    self.buffer.clear();
  }

  // 绘制文本
  pub fn text(&mut self, x: u16, y: u16, text: &str) {
    let mut cursor_x = x;
    for ch in text.chars() {
      if cursor_x >= self.buffer.width() || y >= self.buffer.height() {
        break;
      }
      self.buffer.set(cursor_x, y, CanvasCell::new(ch));
      cursor_x = cursor_x.saturating_add(1);
    }
  }

  // 调整画布大小（resize事件）
  pub fn resize(&mut self, width: u16, height: u16) {
    self.buffer.resize(width, height);
  }

  // 呈现画布
  pub fn present(&mut self, terminal: &mut TerminalService) -> io::Result<()> {
    let Some(stdout) = terminal.writer_mut() else {
      return Ok(());
    };

    stdout.queue(Clear(ClearType::All))?;

    for y in 0..self.buffer.height() {
      stdout.queue(MoveTo(0, y))?;
      stdout.queue(Print(self.buffer.row_text(y)))?;
    }

    stdout.flush()?;
    Ok(())
  }
}
