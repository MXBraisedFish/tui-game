use crate::TextStyle;

/// Single character cell of a canvas: its text, its style and whether it continues a wide
/// character.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasCell {
  pub text: String,
  pub style: TextStyle,
  continuation: bool,
}

impl CanvasCell {
  /// Creates a blank placeholder cell.
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

  /// Creates a styled character cell.
  pub fn styled(text: impl Into<String>, style: TextStyle) -> Self {
    Self {
      text: text.into(),
      style,
      continuation: false,
    }
  }

  /// Creates a wide-character continuation marker, which has no column width of its own.
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn only_continuation_cells_are_marked_and_they_carry_no_text() {
    assert_eq!(CanvasCell::blank().text, " ");
    assert!(!CanvasCell::new("a").is_continuation());
    let continuation = CanvasCell::continuation();
    assert!(continuation.is_continuation());
    assert!(continuation.text.is_empty());
    assert_ne!(continuation, CanvasCell::new(""));
  }
}
