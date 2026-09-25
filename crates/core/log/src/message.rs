use std::borrow::Cow;

/// A host-controlled log message. The key is resolved from the `log_info`
/// runtime namespace and the English fallback is always available during the
/// early boot path or when language loading fails.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostLogMessage {
  pub key: &'static str,
  pub params: Vec<(&'static str, String)>,
  pub english_fallback: &'static str,
}

impl HostLogMessage {
  pub fn new(key: &'static str, english_fallback: &'static str) -> Self {
    Self {
      key,
      params: Vec::new(),
      english_fallback,
    }
  }

  pub fn param(mut self, name: &'static str, value: impl Into<String>) -> Self {
    self.params.push((name, value.into()));
    self
  }

  pub fn render<'a>(&self, template: Option<&'a str>) -> String {
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
