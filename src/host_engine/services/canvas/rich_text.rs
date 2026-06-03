use super::{CanvasBuffer, CanvasStyle, write_text};
use crate::host_engine::services::display_width;
use crate::host_engine::services::rich_text::{RichText, TextStyle};

// 富文本转换为画布样式
fn rich_text_style_to_canvas_style(style: &TextStyle) -> CanvasStyle {
  CanvasStyle {
    foreground: style.foreground.clone(),
    background: style.background.clone(),
    bold: style.bold,
    italic: style.italic,
    underline: style.underline,
    strike: style.strike,
    blink: style.blink,
    reverse: style.reverse,
    hidden: style.hidden,
    dim: style.dim,
  }
}

// 绘制富文本
pub fn write_rich_text(buffer: &mut CanvasBuffer, x: u16, y: u16, rich_text: &RichText) {
  let mut cursor_x = x;

  for segment in &rich_text.segments {
    let canvas_style = rich_text_style_to_canvas_style(&segment.style);

    write_text(buffer, cursor_x, y, &segment.text, canvas_style);

    let segment_width = display_width(&segment.text) as u16;
    cursor_x = cursor_x.saturating_add(segment_width);

    if cursor_x >= buffer.width() {
      break;
    }
  }
}
