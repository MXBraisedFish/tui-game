use std::collections::BTreeSet;
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
  dirty_rows: BTreeSet<u16>,  // 本帧被修改的行
  needs_full_redraw: bool,    // 是否需要全量重绘
}

impl CanvasService {
  pub fn new() -> Self {
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
    Self {
      front_buffer: CanvasBuffer::new(width, height),
      back_buffer: CanvasBuffer::new(width, height),
      dirty_rows: BTreeSet::new(),
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

  // TODO(canvas):
  // 当前运行时每帧都会清空整个后缓冲区，导致所有行都变成脏行。
  // 后续改为保留模式绘制后，应避免不必要的全量清除，以充分发挥脏行优化的效果。
  //
  // 清空画布
  pub fn clear(&mut self) {
    self.back_buffer.clear();
    self.mark_all_rows_dirty();
  }

  // 调整大小
  pub fn resize(&mut self, width: u16, height: u16) {
    self.front_buffer.resize(width, height);
    self.back_buffer.resize(width, height);
    self.needs_full_redraw = true;
    self.mark_all_rows_dirty();
  }

  // 获取尺寸
  pub fn size(&self) -> (u16, u16) {
    (self.back_buffer.width(), self.back_buffer.height())
  }

  // 绘制普通字符
  pub fn write_text(&mut self, x: u16, y: u16, text: &str, style: CanvasStyle) {
    write_text(&mut self.back_buffer, x, y, text, style);
    self.mark_dirty_row(y);
  }

  // 绘制富文本字符
  pub fn write_rich_text(&mut self, x: u16, y: u16, rich_text: &RichText) {
    write_rich_text(&mut self.back_buffer, x, y, rich_text);
    self.mark_dirty_row(y);
  }

  // 提交画布到终端
  pub fn present(&mut self, stdout: &mut Stdout) -> io::Result<()> {
    if self.needs_full_redraw {
      present_buffer(&self.back_buffer, stdout)?;
    } else {
      present_buffer_diff(&self.front_buffer, &self.back_buffer, &self.dirty_rows, stdout)?;
    }

    self.front_buffer.clone_from(&self.back_buffer);
    self.needs_full_redraw = false;
    self.clear_dirty_rows();

    Ok(())
  }

  // 标记单行需要重绘
  fn mark_dirty_row(&mut self, y: u16) {
    if y < self.back_buffer.height() {
      self.dirty_rows.insert(y);
    }
  }

  // 标记所有行需要重绘
  fn mark_all_rows_dirty(&mut self) {
    self.dirty_rows.clear();
    for y in 0..self.back_buffer.height() {
      self.dirty_rows.insert(y);
    }
  }

  // 清空脏行记录
  fn clear_dirty_rows(&mut self) {
    self.dirty_rows.clear();
  }

  // 只读脏行
  pub fn dirty_rows(&self) -> &BTreeSet<u16> {
    &self.dirty_rows
  }

  // 居中绘制普通文本
  pub fn write_centered_text(&mut self, y: u16, text: &str, style: CanvasStyle) {
    write_centered_text(&mut self.back_buffer, y, text, style);
    self.mark_dirty_row(y);
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
