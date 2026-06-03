use super::CanvasStyle;

// 画布网格
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasCell {
  pub ch: char,           // 字符
  pub style: CanvasStyle, // 文本样式
}

impl CanvasCell {
  // 空白内容
  pub fn blank() -> Self {
    Self {
      ch: ' ',
      style: CanvasStyle::default(),
    }
  }

  pub fn new(ch: char, style: CanvasStyle) -> Self {
    Self { ch, style }
  }
}
