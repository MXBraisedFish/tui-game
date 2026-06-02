// 文本样式
#[derive(Clone, Debug, Default)]
pub struct TextStyle {
  pub foreground: Option<TextColor>,
  pub background: Option<TextColor>,
  pub bold: bool,
  pub italic: bool,
  pub underline: bool,
  pub strike: bool,
  pub blink: bool,
  pub reverse: bool,
  pub hidden: bool,
  pub dim: bool,
}

// 颜色
#[derive(Clone, Debug)]
pub enum TextColor {
  Named(String),
  Rgb { r: u8, g: u8, b: u8 },
}
