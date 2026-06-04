use super::{CanvasBuffer, CanvasCell, CanvasStyle};
use crate::host_engine::services::char_width;

/// 绘制文本到画布缓冲区
///
/// 从指定坐标开始逐字写入，自动处理宽字符占位符。
/// 返回实际写入的列宽（光标移动的总距离），
/// 调用方可使用该返回值精确标记脏区间。
pub fn write_text(
  buffer: &mut CanvasBuffer,
  x: u16,
  y: u16,
  text: &str,
  style: CanvasStyle,
) -> u16 {
  // 光标位置
  let mut cursor_x = x;

  // 遍历字符
  for ch in text.chars() {
    // 计算宽度
    let width = char_width(ch);

    // 零宽字符处理
    // TODO(unicode):
    // zero-width characters should eventually attach to previous visible cell.
    if width == 0 {
      // 渲染零宽字符
      buffer.set(cursor_x, y, CanvasCell::character(ch, style.clone()));
      continue; // 跳过移动光标
    }

    // 边界检查
    if cursor_x >= buffer.width() || y >= buffer.height() {
      break;
    }

    // 正常字符（宽度 >= 1）
    buffer.set(cursor_x, y, CanvasCell::character(ch, style.clone()));

    // 占位填充（宽度 > 1）
    for offset in 1..width {
      let next_x = cursor_x.saturating_add(offset as u16);
      if next_x < buffer.width() {
        buffer.set(next_x, y, CanvasCell::wide_continuation(style.clone()));
      }
    }

    cursor_x = cursor_x.saturating_add(width as u16);
  }

  // 返回实际写入的列宽
  cursor_x.saturating_sub(x)
}

