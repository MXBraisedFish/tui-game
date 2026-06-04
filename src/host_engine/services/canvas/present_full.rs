//! 全量画布渲染模块
//!
//! 将整个画布缓冲区的每一个单元格逐行逐列渲染到终端。
//! 适用于首帧渲染或需要完全重绘的场景。

use std::io::{self, Stdout, Write};

use crossterm::QueueableCommand;
use crossterm::cursor::MoveTo;
use crossterm::style::Print;

use super::{apply_canvas_style, reset_canvas_style, CanvasBuffer, CanvasCellContent};

/// 将整个画布缓冲区完整渲染到终端
///
/// 遍历缓冲区的每一行每一列，依次应用样式并输出字符。
/// 对于宽字符占位符（WideContinuation）不做任何打印，因为其显示由前一个宽字符处理。
/// 这是最基础的渲染方式，无任何优化，适合首帧或需要完整重绘的情况。
///
/// - `truecolor` — 是否使用真彩色；false 时降级为 ANSI256
pub fn present_buffer(
  buffer: &CanvasBuffer,
  stdout: &mut Stdout,
  truecolor: bool,
) -> io::Result<()> {
  // 遍历缓冲区每一行
  for y in 0..buffer.height() {
    // 光标移动到当前行开头，确保每行从正确位置开始渲染
    stdout.queue(MoveTo(0, y))?;

    // 遍历当前行的每一列
    for x in 0..buffer.width() {
      // 防御性检查：单元格不存在则跳过
      let Some(cell) = buffer.get(x, y) else {
        continue;
      };

      // 根据单元格内容类型进行不同处理
      match cell.content {
        // 普通字符：应用样式后直接打印
        CanvasCellContent::Character(ch) => {
          apply_canvas_style(stdout, &cell.style, truecolor)?;
          stdout.queue(Print(ch))?;
        }
        // 宽字符占位符：已被前一个宽字符占据，不打印任何内容
        CanvasCellContent::WideContinuation => {}
      }
    }
  }

  // 渲染完成后重置样式并刷新输出缓冲区
  reset_canvas_style(stdout)?;
  stdout.flush()?;

  Ok(())
}
