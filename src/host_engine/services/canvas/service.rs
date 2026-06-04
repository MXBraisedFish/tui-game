use std::io::{self, Stdout};

use super::{
  CanvasBuffer, CanvasStyle, present_buffer, present_buffer_diff, write_centered_text,
  write_rich_text, write_text,
};
use crate::host_engine::services::RichText;

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasService {
  front_buffer: CanvasBuffer, // 上一帧缓冲
  back_buffer: CanvasBuffer,  // 下一帧缓冲
  needs_full_redraw: bool,    // 是否需要全量重绘
}

impl CanvasService {
  pub fn new() -> Self {
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
    Self {
      front_buffer: CanvasBuffer::new(width, height),
      back_buffer: CanvasBuffer::new(width, height),
      needs_full_redraw: true,
    }
  }

  // 只读前缓冲区
  pub fn front_buffer(&self) -> &CanvasBuffer {
    &self.front_buffer
  }

  // 只读后缓存区
  pub fn back_buffer(&self) -> &CanvasBuffer {
    &self.back_buffer
  }

  // 可变访问后缓存区
  pub fn back_buffer_mut(&mut self) -> &mut CanvasBuffer {
    &mut self.back_buffer
  }

  // 清空画布
  pub fn clear(&mut self) {
    self.back_buffer.clear();
  }

  // 调整大小
  pub fn resize(&mut self, width: u16, height: u16) {
    self.front_buffer.resize(width, height);
    self.back_buffer.resize(width, height);
    self.needs_full_redraw = true;
  }

  // 获取尺寸
  pub fn size(&self) -> (u16, u16) {
    (self.back_buffer.width(), self.back_buffer.height())
  }

  // 绘制普通字符
  pub fn write_text(&mut self, x: u16, y: u16, text: &str, style: CanvasStyle) {
    write_text(&mut self.back_buffer, x, y, text, style);
  }

  // 绘制富文本字符
  pub fn write_rich_text(&mut self, x: u16, y: u16, rich_text: &RichText) {
    write_rich_text(&mut self.back_buffer, x, y, rich_text);
  }

  // 提交画布到终端
  pub fn present(&mut self, stdout: &mut Stdout) -> io::Result<()> {
    if self.needs_full_redraw {
      present_buffer(&self.back_buffer, stdout)?;
    } else {
      present_buffer_diff(&self.front_buffer, &self.back_buffer, stdout)?;
    }

    self.front_buffer.clone_from(&self.back_buffer);
    self.needs_full_redraw = false;

    Ok(())
  }

  // 居中绘制普通文本
  pub fn write_centered_text(&mut self, y: u16, text: &str, style: CanvasStyle) {
    write_centered_text(&mut self.back_buffer, y, text, style);
  }

  // 完全重绘
  pub fn needs_full_redraw(&self) -> bool {
    self.needs_full_redraw
  }

  // 临时的行转字符
  pub fn line_as_string(&self, y: u16) -> String {
    self.back_buffer.line_as_string(y)
  }
}
