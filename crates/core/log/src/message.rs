//! Parameterized host log messages with English fallback.

use std::borrow::Cow;

/// A parameterized translation key with an English message fallback available before language
/// loading.
///
/// # Fields
///
/// * `key` - The lookup key.
/// * `params` - The ordered params retained by this owner.
/// * `english_fallback` - The english fallback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostLogMessage {
  /// The lookup key.
  pub key: &'static str,
  /// The ordered params retained by this owner.
  pub params: Vec<(&'static str, String)>,
  /// The english fallback.
  pub english_fallback: &'static str,
}

impl HostLogMessage {
  /// Create a translated log-message key with its always-available English fallback.
  pub fn new(key: &'static str, english_fallback: &'static str) -> Self {
    Self {
      key,
      params: Vec::new(),
      english_fallback,
    }
  }

  /// Attach a named substitution to the host log message and return the updated message.
  pub fn param(mut self, name: &'static str, value: impl Into<String>) -> Self {
    self.params.push((name, value.into()));
    self
  }

  /// Substitute named parameters into a supplied template, or use the embedded English fallback.
  pub fn render(&self, template: Option<&str>) -> String {
    let mut rendered = Cow::Borrowed(template.unwrap_or(self.english_fallback));
    for (name, value) in &self.params {
      rendered = Cow::Owned(rendered.replace(&format!("{{{name}}}"), value));
    }
    rendered.into_owned()
  }
}

#[cfg(test)]
mod tests {
  use super::HostLogMessage;

  #[test]
  fn renders_translated_template_with_parameters() {
    let message = HostLogMessage::new("log_info.test", "Failed: {err}").param("err", "disk full");
    assert_eq!(message.render(Some("失败：{err}")), "失败：disk full");
  }

  #[test]
  fn falls_back_to_embedded_english() {
    let message = HostLogMessage::new("log_info.test", "Ready: {name}").param("name", "engine");
    assert_eq!(message.render(None), "Ready: engine");
  }
}
