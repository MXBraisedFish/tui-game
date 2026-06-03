use super::CanvasStyle;

// 字符宽度
#[derive(Clone, Debug, PartialEq)]
pub enum CanvasCellContent {
  Character(char),  // 普通字符
  WideContinuation, // 宽字符
}

// 画布单元
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasCell {
  pub content: CanvasCellContent, // 内容
  pub style: CanvasStyle,         // 样式
}

impl CanvasCell {
  // 空白单元格
  pub fn blank() -> Self {
    Self {
      content: CanvasCellContent::Character(' '),
      style: CanvasStyle::default(),
    }
  }

  // 普通字符单元格
  pub fn character(ch: char, style: CanvasStyle) -> Self {
    Self {
      content: CanvasCellContent::Character(ch),
      style,
    }
  }

  // 宽字符展位符
  pub fn wide_continuation(style: CanvasStyle) -> Self {
    Self {
      content: CanvasCellContent::WideContinuation,
      style,
    }
  }

  // 是否是占位符
  pub fn is_wide_continuation(&self) -> bool {
    matches!(self.content, CanvasCellContent::WideContinuation)
  }

  // 获取显示字符
  pub fn display_char(&self) -> char {
    match self.content {
      CanvasCellContent::Character(ch) => ch,
      CanvasCellContent::WideContinuation => ' ',
    }
  }
}
