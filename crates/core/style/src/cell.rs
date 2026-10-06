//! Styled terminal graphemes and explicit wide-cell continuation markers.

use crate::TextStyle;

/// A styled grapheme or a trailing-column marker for a wide grapheme.
///
/// # Fields
///
/// * `text` - The text to process or display.
/// * `style` - The text style applied to the rendered content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasCell {
  /// The text to process or display.
  pub text: String,
  /// The text style applied to the rendered content.
  pub style: TextStyle,
  continuation: bool,
}

impl CanvasCell {
  /// Create a blank terminal cell with the default text style.
  pub fn blank() -> Self {
    Self {
      text: " ".to_string(),
      style: TextStyle::default(),
      continuation: false,
    }
  }

  /// Create a text cell with the default style and no wide-cell continuation marker.
  pub fn new(text: impl Into<String>) -> Self {
    Self {
      text: text.into(),
      style: TextStyle::default(),
      continuation: false,
    }
  }

  /// Create a text cell carrying the supplied terminal style.
  pub fn styled(text: impl Into<String>, style: TextStyle) -> Self {
    Self {
      text: text.into(),
      style,
      continuation: false,
    }
  }

  /// Create the marker occupying the trailing column of a wide grapheme.
  pub fn continuation() -> Self {
    Self {
      text: String::new(),
      style: TextStyle::default(),
      continuation: true,
    }
  }
  /// Report whether this cell is the trailing marker of a wide grapheme.
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
