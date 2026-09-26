use super::style::TextStyle;

/// Parsed rich text: a sequence of styled text segments.
#[derive(Clone, Debug)]
pub struct RichText {
  pub segments: Vec<RichTextSegment>,
}

/// One styled segment of rich text: its text and the [`TextStyle`] applied to it.
#[derive(Clone, Debug)]
pub struct RichTextSegment {
  pub text: String,
  pub style: TextStyle,
}
