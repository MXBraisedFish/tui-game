//! Service support for the rich text service.

use super::{RichText, RichTextParams, parser};

/// The plain, explicit rich, or prefix-triggered automatic parsing mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextMode {
  /// The auto setting for text mode.
  #[default]
  Auto,
  /// The plain setting for text mode.
  Plain,
  /// The rich setting for text mode.
  Rich,
}

/// The public entry point for rich text operations.
pub struct RichTextService;

impl RichTextService {
  /// Create a rich text service with its initial state.
  pub fn new() -> Self {
    Self
  }

  /// Parse host text, explicitly enabling substitutions when parameters are supplied.
  pub fn parse(&self, text: &str, params: Option<&RichTextParams>) -> RichText {
    parser::parse_auto(text, params)
  }

  /// Parse literal or tagged text according to the explicitly selected mode.
  ///
  /// # Arguments
  ///
  /// * `text` - The text to process or display.
  /// * `params` - The formatting or rendering parameters.
  /// * `mode` - The mode.
  pub fn parse_mode(
    &self,
    text: &str,
    params: Option<&RichTextParams>,
    mode: TextMode,
  ) -> RichText {
    match mode {
      TextMode::Auto => parser::parse_auto(text, params),
      TextMode::Plain => parser::parse_plain(text),
      TextMode::Rich => parser::parse_rich(text, params),
    }
  }

  /// Return rendered text with formatting directives removed.
  pub fn visible_text(&self, text: &str, params: Option<&RichTextParams>) -> String {
    if params.is_none() && !text.starts_with("f%") {
      return text.to_string();
    }

    // Host parameters explicitly request formatting; Lua AUTO text still needs its f% prefix.

    let rich_text = if params.is_some() {
      parser::parse_rich(text.strip_prefix("f%").unwrap_or(text), params)
    } else {
      self.parse(text, params)
    };
    let mut result = String::new();
    for segment in &rich_text.segments {
      result.push_str(&segment.text);
    }
    result
  }
}

impl Default for RichTextService {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn host_visible_text_resolves_unprefixed_parameters() {
    let mut params = RichTextParams::default();
    params.values.insert("action".into(), "[Enter]".into());

    let rich = RichTextService::new();
    assert_eq!(
      rich.visible_text("{value:action}", Some(&params)),
      "[Enter]"
    );
    assert_eq!(
      rich.visible_text("f%{value:action}", Some(&params)),
      "[Enter]"
    );
  }
}
