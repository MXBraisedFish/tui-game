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
    //
    // 当前不支持组合标记、字形簇、表情符号组合等复杂 Unicode 结构。
    // 零宽字符跳过渲染以保持画布一致性（不写缓冲区、不移动光标、不标记脏区间）。
    //
    // TODO(Canvas Unicode Phase 2): 完整的字形簇渲染
    //   目标: 迭代字形簇而非单个字符
    //   所需工作:
    //     1. 添加 unicode-segmentation 依赖
    //     2. 迭代 .graphemes(true) 替代 .chars()
    //     3. 添加 grapheme_width() 替代 char_width()
    //     4. CanvasCell 从单字符存储改为字形字符串存储
    //     5. 呈现器打印字形字符串而非单字符
    //     6. 差异渲染器比较字形内容
    //     7. 表情符号 ZWJ 序列支持
    //     8. 组合标记支持（e + ◌́ → é 视为一个单元）
    //
    //   当前架构:  char → char_width() → CanvasCell(char)
    //   未来架构:  grapheme cluster → grapheme_width() → CanvasCell(grapheme_str)
    //
    // 当前支持范围:
    //   ✅ ASCII / Unicode 标量值 / CJK 宽字符
    //   ❌ 组合标记 / 字形簇 / 表情符号组合 / ZWJ 序列
    if width == 0 {
      continue; // 不写入缓冲区，不移动光标，不标记脏区间
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

