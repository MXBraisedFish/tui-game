/// Grapheme cluster with its text and terminal display width.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphemeInfo {
  pub text: String,

  pub display_width: usize,
}
