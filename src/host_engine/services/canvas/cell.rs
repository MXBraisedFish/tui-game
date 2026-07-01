use crate::host_engine::services::TextStyle;

/// 画布上的单个字符单元，包含文本内容、样式和是否为宽字符延续标记。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasCell {
  pub text: String,
  pub style: TextStyle,
  continuation: bool,
}

impl CanvasCell {
  /// 创建一个空白占位单元格。
  pub fn blank() -> Self {
    Self {
      text: " ".to_string(),
      style: TextStyle::default(),
      continuation: false,
    }
  }

  pub fn new(text: impl Into<String>) -> Self {
    Self {
      text: text.into(),
      style: TextStyle::default(),
      continuation: false,
    }
  }

  /// 创建一个带样式的字符单元格。
  pub fn styled(text: impl Into<String>, style: TextStyle) -> Self {
    Self {
      text: text.into(),
      style,
      continuation: false,
    }
  }

  /// 创建一个宽字符延续标记（不占独立列宽）。
  pub fn continuation() -> Self {
    Self {
      text: String::new(),
      style: TextStyle::default(),
      continuation: true,
    }
  }
  pub fn is_continuation(&self) -> bool {
    self.continuation
  }
}
