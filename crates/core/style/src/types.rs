//! Data shared by the independent core.

use super::style::TextStyle;

/// A sequence of text segments carrying resolved styles.
///
/// # Fields
///
/// * `segments` - The ordered segments retained by this owner.
#[derive(Clone, Debug)]
pub struct RichText {
  /// The ordered segments retained by this owner.
  pub segments: Vec<RichTextSegment>,
}

/// A text segment with one resolved terminal style.
///
/// # Fields
///
/// * `text` - The text to process or display.
/// * `style` - The text style applied to the rendered content.
#[derive(Clone, Debug)]
pub struct RichTextSegment {
  /// The text to process or display.
  pub text: String,
  /// The text style applied to the rendered content.
  pub style: TextStyle,
}
