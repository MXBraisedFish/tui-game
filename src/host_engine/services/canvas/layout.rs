use super::{CanvasBuffer, CanvasStyle, write_text};
use crate::host_engine::services::display_width;

// 临时居中绘制
pub fn write_centered_text(buffer: &mut CanvasBuffer, y: u16, text: &str, style: CanvasStyle) {
  if y >= buffer.height() {
    return;
  }

  let text_width = display_width(text) as u16;

  let x = if text_width < buffer.width() {
    (buffer.width() - text_width) / 2
  } else {
    0
  };

  write_text(buffer, x, y, text, style);
}
