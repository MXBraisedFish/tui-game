use crossterm::style::Attribute;
use std::io::{self, Stdout, Write};

use crossterm::QueueableCommand;
use crossterm::cursor::MoveTo;
use crossterm::style::{Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor};

use super::{
  CanvasBuffer, CanvasCellContent, CanvasStyle, style_attributes, text_color_to_crossterm_color,
};

// 应用画布样式
fn apply_canvas_style(stdout: &mut Stdout, style: &CanvasStyle) -> io::Result<()> {
  stdout.queue(ResetColor)?;

  if let Some(foreground) = &style.foreground {
    stdout.queue(SetForegroundColor(text_color_to_crossterm_color(
      foreground,
    )))?;
  }

  if let Some(background) = &style.background {
    stdout.queue(SetBackgroundColor(text_color_to_crossterm_color(
      background,
    )))?;
  }

  for attribute in style_attributes(style) {
    stdout.queue(SetAttribute(attribute))?;
  }

  Ok(())
}

// 重置画布样式
fn reset_canvas_style(stdout: &mut Stdout) -> io::Result<()> {
  stdout.queue(ResetColor)?;
  stdout.queue(SetAttribute(Attribute::Reset))?;
  Ok(())
}

// 将缓冲区会知到终端上
pub fn present_buffer(buffer: &CanvasBuffer, stdout: &mut Stdout) -> io::Result<()> {
  // 遍历缓冲区每一行
  for y in 0..buffer.height() {
    // 光标移动到当前行开头
    stdout.queue(MoveTo(0, y))?;

    // 遍历每一列
    for x in 0..buffer.width() {
      // 单元格不存在就跳过（防御）
      let Some(cell) = buffer.get(x, y) else {
        continue;
      };

      // 根据类型输出
      match cell.content {
        // 正常字符直接输出
        CanvasCellContent::Character(ch) => {
          apply_canvas_style(stdout, &cell.style)?;
          stdout.queue(Print(ch))?;
        }
        CanvasCellContent::WideContinuation => {
          // 已经被前一个宽字符占据，这里不打印任何东西
        }
      }
    }
  }

  // 刷新缓冲区
  reset_canvas_style(stdout)?;
  stdout.flush()?;
  Ok(())
}
