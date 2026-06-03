use super::{CanvasBuffer, CanvasCell, CanvasStyle};
use crate::host_engine::services::char_width;

// 绘制文本
pub fn write_text(buffer: &mut CanvasBuffer, x: u16, y: u16, text: &str, style: CanvasStyle) {
  // 光标位置
  let mut cursor_x = x;

  // 遍历字符
  for ch in text.chars() {
    // 计算宽度
    let width = char_width(ch);

    // 零宽字符处理
    if width == 0 {
      // 渲染零宽字符
      buffer.set(cursor_x, y, CanvasCell::new(ch, style.clone()));
      continue; // 跳过移动光标
    }

    // 边界检查
    if cursor_x >= buffer.width() || y >= buffer.height() {
      break;
    }

    // 正常字符（宽度 >= 1）
    buffer.set(cursor_x, y, CanvasCell::new(ch, style.clone()));

    // 占位填充（宽度 > 1）
    for offset in 1..width {
      let next_x = cursor_x.saturating_add(offset as u16);
      if next_x < buffer.width() {
        buffer.set(next_x, y, CanvasCell::blank());
      }
    }

    cursor_x = cursor_x.saturating_add(width as u16);
  }
}
