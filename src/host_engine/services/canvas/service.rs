use super::{CanvasBuffer, CanvasStyle, write_text};

pub struct CanvasService {
  back_buffer: CanvasBuffer,
}

impl CanvasService {
  pub fn new() -> Self {
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
    Self {
      back_buffer: CanvasBuffer::new(width, height),
    }
  }

  // 只读缓存区
  pub fn back_buffer(&self) -> &CanvasBuffer {
    &self.back_buffer
  }

  // 可变访问
  pub fn back_buffer_mut(&mut self) -> &mut CanvasBuffer {
    &mut self.back_buffer
  }

  // 清空画布
  pub fn clear(&mut self) {
    self.back_buffer.clear();
  }

  // 调整大小
  pub fn resize(&mut self, width: u16, height: u16) {
    self.back_buffer.resize(width, height);
  }

  // 获取尺寸
  pub fn size(&self) -> (u16, u16) {
    (self.back_buffer.width(), self.back_buffer.height())
  }

  // 绘制普通字符
  pub fn write_text(&mut self, x: u16, y: u16, text: &str, style: CanvasStyle) {
    write_text(&mut self.back_buffer, x, y, text, style);
  }

  // 临时的行转字符
  pub fn line_as_string(&self, y: u16) -> String {
    self.back_buffer.line_as_string(y)
  }
}
