use crate::host_engine::services::TextStyle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasCell {
  /// 当前单元格起始位置保存的完整 grapheme cluster。
  pub text: String,
  pub style: TextStyle,
  continuation: bool,
}

impl CanvasCell {
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

  pub fn styled(text: impl Into<String>, style: TextStyle) -> Self {
    Self {
      text: text.into(),
      style,
      continuation: false,
    }
  }

  /// 构造一个"宽字符延续"占位格。
  pub fn continuation() -> Self {
    Self {
      text: String::new(),
      style: TextStyle::default(),
      continuation: true,
    }
  }

  /// 是否为"宽字符延续"占位格。
  pub fn is_continuation(&self) -> bool {
    self.continuation
  }
}
