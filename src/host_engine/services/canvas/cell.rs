use crate::host_engine::services::TextStyle;

/// 标记被左侧宽字符"占用"的单元格。
/// 终端中宽字符（如 CJK、emoji）占 2 列，
/// 它右侧的那一格不写入独立字符，仅作为视觉延续。
pub const WIDE_CONTINUATION: char = '\0';

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanvasCellContent {
  Text(char),
  Raw(String),
  Skip,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasCell {
  pub content: CanvasCellContent,
  pub style: TextStyle,
}

impl CanvasCell {
  pub fn blank() -> Self {
    Self {
      content: CanvasCellContent::Text(' '),
      style: TextStyle::default(),
    }
  }

  pub fn new(ch: char) -> Self {
    Self {
      content: CanvasCellContent::Text(ch),
      style: TextStyle::default(),
    }
  }

  pub fn styled(ch: char, style: TextStyle) -> Self {
    Self {
      content: CanvasCellContent::Text(ch),
      style,
    }
  }

  pub fn raw(seq: String) -> Self {
    Self {
      content: CanvasCellContent::Raw(seq),
      style: TextStyle::default(),
    }
  }

  pub fn skip() -> Self {
    Self {
      content: CanvasCellContent::Skip,
      style: TextStyle::default(),
    }
  }

  pub fn text_char(&self) -> Option<char> {
    match &self.content {
      CanvasCellContent::Text(ch) => Some(*ch),
      CanvasCellContent::Raw(_) | CanvasCellContent::Skip => None,
    }
  }

  /// 构造一个"宽字符延续"占位格。
  pub fn continuation() -> Self {
    Self {
      content: CanvasCellContent::Text(WIDE_CONTINUATION),
      style: TextStyle::default(),
    }
  }

  /// 是否为"宽字符延续"占位格。
  pub fn is_continuation(&self) -> bool {
    self.text_char() == Some(WIDE_CONTINUATION)
  }

  pub fn is_skip(&self) -> bool {
    matches!(self.content, CanvasCellContent::Skip)
  }
}
