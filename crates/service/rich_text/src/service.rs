use super::{RichText, RichTextParams, parser};

/// Text parsing mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextMode {
  #[default]
  Auto,
  Plain,
  Rich,
}

/// Rich text service that parses rich text and extracts its visible plain text.
pub struct RichTextService;

impl RichTextService {
  pub fn new() -> Self {
    Self
  }

  /// Parses a rich text string into a list of styled segments.
  ///
  /// Uses [`TextMode::Auto`]: only text with the `f%` prefix is formatted; other text stays plain.
  pub fn parse(&self, text: &str, params: Option<&RichTextParams>) -> RichText {
    parser::parse_auto(text, params)
  }

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

  /// Parses rich text and returns only its visible text (all style tags removed).
  pub fn visible_text(&self, text: &str, params: Option<&RichTextParams>) -> String {
    if params.is_none() && !text.starts_with("f%") {
      return text.to_string();
    }

    // Host UIs that pass parameters have explicitly asked for formatting; Lua's AUTO mode still
    // requires the `f%` prefix.
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
