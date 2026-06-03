use std::io::{self, Stdout, Write};

use crossterm::QueueableCommand;
use crossterm::cursor::MoveTo;
use crossterm::style::Print;

use super::{CanvasBuffer, CanvasCellContent};

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
          stdout.queue(Print(ch))?;
        }
        CanvasCellContent::WideContinuation => {
          // 已经被前一个宽字符占据，这里不打印任何东西
        }
      }
    }
  }

  // 刷新缓冲区
  stdout.flush()?;
  Ok(())
}
