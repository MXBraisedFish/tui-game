#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasCell {
  pub ch: char,
}

impl CanvasCell {
  // 空白单元格
  pub fn blank() -> Self {
    Self { ch: ' ' }
  }

  // 创建单元格
  pub fn new(ch: char) -> Self {
    Self { ch }
  }
}
