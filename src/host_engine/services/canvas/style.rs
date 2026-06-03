use crate::host_engine::services::rich_text::{TerminalColor, TextColor};

// 画布样式表
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasStyle {
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

impl CanvasStyle {
  pub fn default_style() -> Self {
    Self::default()
  }
}
