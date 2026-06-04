use std::io::{self, Stdout};

use super::{
  CanvasBuffer, CanvasStyle, DirtySpan, present_buffer, present_buffer_diff, write_rich_text,
  write_text,
};
use crate::host_engine::services::{RichText, TerminalService};

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasService {
  front_buffer: CanvasBuffer, // 上一帧缓冲
  back_buffer: CanvasBuffer,  // 下一帧缓冲
  dirty_spans: Vec<DirtySpan>, // 本帧被修改的区域
  needs_full_redraw: bool,    // 是否需要全量重绘
  truecolor: bool,            // 是否使用真彩色（false 降级为 ANSI256）
  is_drawing_frame: bool,     // 是否正在绘制帧
}

impl CanvasService {
  pub fn new() -> Self {
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
    Self {
      front_buffer: CanvasBuffer::new(width, height),
      back_buffer: CanvasBuffer::new(width, height),
      dirty_spans: Vec::new(),
      needs_full_redraw: true,
      truecolor: true,
      is_drawing_frame: false,
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

  // TODO(canvas):
  // 当前运行时每帧都会清空整个后缓冲区，导致所有行都变成脏行。
  // 后续改为保留模式绘制后，应避免不必要的全量清除，以充分发挥脏行优化的效果。
  //
  // 清空画布
  pub fn clear(&mut self) {
    self.back_buffer.clear();
    self.mark_all_dirty();
  }

  // 清理指定行
  pub fn clear_row(&mut self, y: u16) {
    self.back_buffer.clear_row(y);
    self.mark_dirty_line(y);
  }

  // 清理指定水平区间
  //
  // 仅清除一行中的 [start_x, end_x) 列范围，
  // 脏区间标记与实际清除范围一致，不会扩大为整行。
  // 适合替换有界文本区域（如标签、数值显示）时使用。
  pub fn clear_span(&mut self, y: u16, start_x: u16, end_x: u16) {
    self.back_buffer.clear_span(y, start_x, end_x);
    self.mark_dirty_span(y, start_x, end_x);
  }

  // 开始新帧
  //
  // 标记进入绘制状态，不清空缓冲区（保留模式）。
  // 绘制代码自行负责清除需要重写的行/区域。
  // 后续可扩展为处理脏矩形重置、帧标记、绘制命令队列重置等。
  pub fn begin_frame(&mut self) {
    self.is_drawing_frame = true;
  }

  // 调整大小
  pub fn resize(&mut self, width: u16, height: u16) {
    self.front_buffer.resize(width, height);
    self.back_buffer.resize(width, height);
    self.needs_full_redraw = true;
    self.mark_all_dirty();
  }

  // 获取尺寸
  pub fn size(&self) -> (u16, u16) {
    (self.back_buffer.width(), self.back_buffer.height())
  }

  // 绘制普通字符
  pub fn write_text(&mut self, x: u16, y: u16, text: &str, style: CanvasStyle) {
    let width = write_text(&mut self.back_buffer, x, y, text, style);
    self.mark_dirty_span(y, x, x.saturating_add(width));
  }

  // 绘制富文本字符
  pub fn write_rich_text(&mut self, x: u16, y: u16, rich_text: &RichText) {
    let width = write_rich_text(&mut self.back_buffer, x, y, rich_text);
    self.mark_dirty_span(y, x, x.saturating_add(width));
  }

  // 提交画布到终端
  pub fn present(&mut self, stdout: &mut Stdout) -> io::Result<()> {
    if self.needs_full_redraw {
      present_buffer(&self.back_buffer, stdout, self.truecolor)?;
    } else {
      present_buffer_diff(
        &self.front_buffer,
        &self.back_buffer,
        &self.dirty_spans,
        stdout,
        self.truecolor,
      )?;
    }

    self.front_buffer.clone_from(&self.back_buffer);
    self.needs_full_redraw = false;
    self.clear_dirty_spans();
    self.finish_frame();

    Ok(())
  }

  // 结束当前帧
  //
  // 由 present() 成功后自动调用，标记绘制状态结束。
  fn finish_frame(&mut self) {
    self.is_drawing_frame = false;
  }

  // 请求帧更新
  //
  // 封装终端交互逻辑：
  // 1. 从 TerminalService 获取真彩色能力并同步
  // 2. 获取终端 writer 并调用 present() 提交画布
  pub fn request_frame_update(
    &mut self,
    terminal: &mut TerminalService,
  ) -> io::Result<()> {
    // 同步终端的真彩色能力到画布
    self.set_truecolor(terminal.capabilities().truecolor);

    // 获取终端 writer；若终端未激活，仍需关闭帧生命周期
    let Some(stdout) = terminal.writer_mut() else {
      self.finish_frame();
      return Ok(());
    };

    self.present(stdout)
  }

  // 是否正在绘制帧
  pub fn is_drawing_frame(&self) -> bool {
    self.is_drawing_frame
  }

  // 标记指定行的脏区间
  //
  // 自动将 start_x 和 end_x 规范化为缓冲区边界内的有效范围，
  // 确保下游的差异渲染始终接收干净的数据。
  fn mark_dirty_span(&mut self, y: u16, start_x: u16, end_x: u16) {
    if y >= self.back_buffer.height() {
      return;
    }

    let buffer_width = self.back_buffer.width();

    // 将坐标钳制在缓冲区范围内，防止越界
    let normalized_start_x = start_x.min(buffer_width);
    let normalized_end_x = end_x.min(buffer_width);

    let new_span = DirtySpan::new(y, normalized_start_x, normalized_end_x);

    if new_span.is_empty() {
      return;
    }

    // 尝试与现有同行的区间合并，避免冗余记录
    for span in &mut self.dirty_spans {
      if span.merge_if_possible(new_span) {
        return;
      }
    }

    self.dirty_spans.push(new_span);
  }

  // 标记整行需要重绘
  fn mark_dirty_line(&mut self, y: u16) {
    if y >= self.back_buffer.height() {
      return;
    }
    self.mark_dirty_span(y, 0, self.back_buffer.width());
  }

  // 标记所有行需要重绘
  fn mark_all_dirty(&mut self) {
    self.dirty_spans.clear();
    for y in 0..self.back_buffer.height() {
      self.dirty_spans.push(DirtySpan::new(y, 0, self.back_buffer.width()));
    }
  }

  // 清空脏区间记录
  fn clear_dirty_spans(&mut self) {
    self.dirty_spans.clear();
  }

  // 只读脏区间列表
  pub fn dirty_spans(&self) -> &[DirtySpan] {
    &self.dirty_spans
  }

  // 完全重绘
  pub fn needs_full_redraw(&self) -> bool {
    self.needs_full_redraw
  }

  // 设置是否使用真彩色（false 时 RGB 降级为 ANSI256）
  pub fn set_truecolor(&mut self, truecolor: bool) {
    self.truecolor = truecolor;
  }

  // 是否正在使用真彩色
  pub fn truecolor(&self) -> bool {
    self.truecolor
  }
}
