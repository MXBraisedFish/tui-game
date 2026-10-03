//! Service support for the i18n service.

use std::collections::HashMap;

use super::{LanguageInfo, LanguageRegistryEntry};

const HARD_CODED_MISSING_TEMPLATE: &str = "[Missing i18n Key: {value:missing_key}]";

/// The public entry point for i18n operations.
pub struct I18nService {
  current_language: String,
  current_language_info: Option<LanguageInfo>,
  language_registry: Vec<LanguageRegistryEntry>,
  runtime_texts: HashMap<String, HashMap<String, String>>,
}

impl Default for I18nService {
  fn default() -> Self {
    Self::new()
  }
}

impl I18nService {
  /// Create an i18n service with its initial state.
  pub fn new() -> Self {
    Self {
      current_language: String::new(),
      current_language_info: None,
      language_registry: Vec::new(),
      runtime_texts: HashMap::new(),
    }
  }

  /// Return the current language.
  pub fn current_language(&self) -> &str {
    &self.current_language
  }

  /// Return the current language code.
  pub fn current_language_code(&self) -> &str {
    &self.current_language
  }

  /// Update the current language used by this i18n service.
  pub fn set_current_language(&mut self, language_code: impl Into<String>) {
    self.current_language = language_code.into();
  }

  /// Clear the runtime texts retained by this i18n service.
  pub fn clear_runtime_texts(&mut self) {
    self.runtime_texts.clear();
  }

  /// Report whether this i18n service is runtime empty.
  pub fn is_runtime_empty(&self) -> bool {
    self.runtime_texts.is_empty()
  }

  /// Replace a runtime namespace with the supplied translation values.
  pub fn insert_runtime_namespace(
    &mut self,
    namespace: impl Into<String>,
    texts: HashMap<String, String>,
  ) {
    self.runtime_texts.insert(namespace.into(), texts);
  }

  /// Add translation values while preserving keys already present in the namespace.
  pub(super) fn merge_runtime_namespace(
    &mut self,
    namespace: impl Into<String>,
    texts: HashMap<String, String>,
  ) {
    let namespace = self.runtime_texts.entry(namespace.into()).or_default();
    for (key, value) in texts {
      namespace.entry(key).or_insert(value);
    }
  }

  /// Resolve a namespaced runtime translation key.
  pub fn get_runtime_text(&self, namespace: &str, key: &str) -> String {
    if let Some(text) = self
      .runtime_texts
      .get(namespace)
      .and_then(|texts| texts.get(key))
      .cloned()
    {
      return text;
    }

    let missing_key = if key.starts_with(&format!("{namespace}.")) {
      key.to_string()
    } else {
      format!("{namespace}.{key}")
    };
    let template = self
      .runtime_texts
      .get("language_warning")
      .and_then(|texts| texts.get("language_warning.missing"))
      .map(String::as_str)
      .unwrap_or(HARD_CODED_MISSING_TEMPLATE);
    if namespace == "language_warning" && key == "language_warning.missing" {
      return template.to_string();
    }
    template.replace("{value:missing_key}", &missing_key)
  }

  /// Return the translation map stored under the requested runtime namespace.
  pub fn runtime_namespace(&self, namespace: &str) -> Option<&HashMap<String, String>> {
    self.runtime_texts.get(namespace)
  }

  /// Return the current language info.
  pub fn current_language_info(&self) -> Option<&LanguageInfo> {
    self.current_language_info.as_ref()
  }

  /// Update the current language info used by this i18n service.
  pub fn set_current_language_info(&mut self, info: Option<LanguageInfo>) {
    self.current_language_info = info;
  }

  /// Return the current language registry.
  pub fn language_registry(&self) -> &[LanguageRegistryEntry] {
    &self.language_registry
  }

  /// Update the language registry used by this i18n service.
  pub fn set_language_registry(&mut self, registry: Vec<LanguageRegistryEntry>) {
    self.language_registry = registry;
  }

  /// Report whether the language code is present in the current registry.
  pub fn is_registered_language(&self, language_code: &str) -> bool {
    self
      .language_registry
      .iter()
      .any(|entry| entry.code == language_code)
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use super::I18nService;

  #[test]
  fn current_language_code_returns_active_language_code() {
    let mut service = I18nService::new();
    service.set_current_language("zh_cn");

    assert_eq!(service.current_language_code(), "zh_cn");
  }

  #[test]
  fn fallback_merge_fills_only_missing_keys() {
    let mut service = I18nService::new();
    service.insert_runtime_namespace(
      "screen",
      HashMap::from([("screen.current".to_string(), "当前语言".to_string())]),
    );
    service.merge_runtime_namespace(
      "screen",
      HashMap::from([
        ("screen.current".to_string(), "English".to_string()),
        ("screen.fallback".to_string(), "Fallback".to_string()),
      ]),
    );

    assert_eq!(
      service.get_runtime_text("screen", "screen.current"),
      "当前语言"
    );
    assert_eq!(
      service.get_runtime_text("screen", "screen.fallback"),
      "Fallback"
    );
  }

  #[test]
  fn missing_key_uses_language_warning_then_hard_coded_template() {
    let mut service = I18nService::new();
    service.insert_runtime_namespace(
      "language_warning",
      HashMap::from([(
        "language_warning.missing".to_string(),
        "[缺少：{value:missing_key}]".to_string(),
      )]),
    );
    assert_eq!(
      service.get_runtime_text("recording_list", "recording_list.action.unknown"),
      "[缺少：recording_list.action.unknown]"
    );

    service.clear_runtime_texts();
    assert_eq!(
      service.get_runtime_text("language_warning", "language_warning.missing"),
      "[Missing i18n Key: {value:missing_key}]"
    );
    assert_eq!(
      service.get_runtime_text("recording_list", "action.unknown"),
      "[Missing i18n Key: recording_list.action.unknown]"
    );
  }
}
