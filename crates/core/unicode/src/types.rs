//! Data shared by the independent core.

/// One Unicode grapheme and its terminal display width.
///
/// # Fields
///
/// * `text` - The text to process or display.
/// * `display_width` - The occupied width in terminal columns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphemeInfo {
  /// The text to process or display.
  pub text: String,

  /// The occupied width in terminal columns.
  pub display_width: usize,
}
