use std::collections::HashMap;

use super::{LanguageInfo, LanguageRegistryEntry};

const HARD_CODED_MISSING_TEMPLATE: &str = "[Missing i18n Key: {value:missing_key}]";

/// 国际化服务，管理多语言文本和语言注册表
pub struct I18nService {
  current_language: String,
  current_language_info: Option<LanguageInfo>,
  language_registry: Vec<LanguageRegistryEntry>,
  runtime_texts: HashMap<String, HashMap<String, String>>,
}

impl I18nService {
  pub fn new() -> Self {
    Self {
      current_language: String::new(),
      current_language_info: None,
      language_registry: Vec::new(),
      runtime_texts: HashMap::new(),
    }
  }

  pub fn current_language(&self) -> &str {
    &self.current_language
  }

  pub fn current_language_code(&self) -> &str {
    &self.current_language
  }

  pub fn set_current_language(&mut self, language_code: impl Into<String>) {
    self.current_language = language_code.into();
  }

  pub fn clear_runtime_texts(&mut self) {
    self.runtime_texts.clear();
  }

  /// 检查运行时文本是否为空
  pub fn is_runtime_empty(&self) -> bool {
    self.runtime_texts.is_empty()
  }

  pub fn insert_runtime_namespace(
    &mut self,
    namespace: impl Into<String>,
    texts: HashMap<String, String>,
  ) {
    self.runtime_texts.insert(namespace.into(), texts);
  }

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

  /// 获取指定命名空间下的运行时翻译文本，未找到时返回本地化的缺失标记。
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

  pub fn current_language_info(&self) -> Option<&LanguageInfo> {
    self.current_language_info.as_ref()
  }

  pub fn set_current_language_info(&mut self, info: Option<LanguageInfo>) {
    self.current_language_info = info;
  }

  pub fn language_registry(&self) -> &[LanguageRegistryEntry] {
    &self.language_registry
  }

  pub fn set_language_registry(&mut self, registry: Vec<LanguageRegistryEntry>) {
    self.language_registry = registry;
  }

  /// 检查指定语言代码是否在注册表中
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
