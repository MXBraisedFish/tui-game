//! 画布样式工具模块
//!
//! 提供将内部画布样式应用到终端输出的函数。
//! 这些函数被全量渲染和差异渲染共享使用。

use crossterm::style::Attribute;
use std::io::{self, Stdout};

use crossterm::QueueableCommand;
use crossterm::style::{ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor};

use super::{style_attributes, text_color_to_crossterm_color, CanvasStyle};

/// 将画布样式应用到终端输出
///
/// 依次设置前景色、背景色以及各种文字属性（粗体、斜体等）。
/// 在应用新样式前会先重置当前样式，避免样式残留。
///
/// - `truecolor` — 是否使用真彩色；false 时降级为 ANSI256
pub fn apply_canvas_style(
  stdout: &mut Stdout,
  style: &CanvasStyle,
  truecolor: bool,
) -> io::Result<()> {
  // 先重置所有颜色，确保新样式不被旧样式污染
  stdout.queue(ResetColor)?;

  // 设置前景色（文字颜色）
  if let Some(foreground) = &style.foreground {
    stdout.queue(SetForegroundColor(text_color_to_crossterm_color(
      foreground,
      truecolor,
    )))?;
  }

  // 设置背景色
  if let Some(background) = &style.background {
    stdout.queue(SetBackgroundColor(text_color_to_crossterm_color(
      background,
      truecolor,
    )))?;
  }

  // 设置文字属性（粗体、斜体、下划线等）
  for attribute in style_attributes(style) {
    stdout.queue(SetAttribute(attribute))?;
  }

  Ok(())
}

/// 重置画布样式到终端默认状态
///
/// 清除所有颜色和文字属性，恢复到终端默认外观。
/// 通常在每帧渲染开始时或完成时调用，防止样式泄漏到下一帧。
pub fn reset_canvas_style(stdout: &mut Stdout) -> io::Result<()> {
  stdout.queue(ResetColor)?;
  stdout.queue(SetAttribute(Attribute::Reset))?;
  Ok(())
}
