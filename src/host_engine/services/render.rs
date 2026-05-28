// 引入官方标准输入输出库
use std::io::{self, stdout, Write};

// 光标移动
use crossterm::cursor::MoveTo;
// 打印
use crossterm::style::Print;
// 清屏
use crossterm::terminal::{Clear, ClearType};
// 命令队列
use crossterm::QueueableCommand;

pub struct RenderService {
  width: u16, // 终端字符宽度
  height: u16, // 终端字符高度
  lines: Vec<String> // 帧缓冲区
}

impl RenderService {
  pub fn new() -> Self {
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));

    Self {
      width,
      height,
      lines: Vec::new()
    }
  }

  // 清理缓冲区
  pub fn clear(&mut self) {
    self.lines.clear();
  }

  // 居中绘制
  pub fn draw_centered(&mut self, row: usize, text: &str) {
    // 计算字符宽度(这里是字符数)
    // TODO：应当使用Unicode宽度
    let text_width = text.chars().count() as u16;

    // 水平居中算法
    // 若小于终端长度，则计算，否则直接从边缘开始
    let col = if text_width < self.width {
      (self.width - text_width) / 2
    } else {
      0
    };

    // 确保行存在，没有就补充空行
    // 注意0行也需要填充
    while self.lines.len() <= row {
      self.lines.push(String::new());
    }

    // 创建左空格用于边距
    let padding = " ".repeat(col as usize);
    // 组合边距和内容
    self.lines[row] = format!("{}{}", padding, text);
  }

  pub fn present(&mut self) -> io::Result<()> {
    // 获取终端输出流控制权
    let mut stdout = stdout();

    // 强制移动光标
    stdout.queue(MoveTo(0, 0))?;
    // 清空屏幕
    stdout.queue(Clear(ClearType::All))?;

    // 遍历数组并绘制每一行
    for (row, line) in self.lines.iter().enumerate() {
      // 避免超出行数（高度）
      // 超出的内容直接丢弃
      if row >= self.height as usize {
        break;
      }

      // 将光标移动到行的最左侧，入队列
      stdout.queue(MoveTo(0, row as u16))?;
      // 在光标位置开始打印字符，入队列
      stdout.queue(Print(line.as_str()))?;
    }

    // 刷新输出缓冲区
    stdout.flush()?;

    Ok(())
  }

  // 获取初始存储的终端宽高
  pub fn size(&self) -> (u16, u16) {
    (self.width, self.height)
  }
}