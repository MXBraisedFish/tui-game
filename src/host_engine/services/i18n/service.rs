use std::collections::HashMap;

use super::{LanguageInfo, LanguageRegistryEntry};

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

  pub fn set_current_language(&mut self, language_code: impl Into<String>) {
    self.current_language = language_code.into();
  }

  pub fn clear_runtime_texts(&mut self) {
    self.runtime_texts.clear();
  }

  pub fn insert_runtime_namespace(
    &mut self,
    namespace: impl Into<String>,
    texts: HashMap<String, String>,
  ) {
    self.runtime_texts.insert(namespace.into(), texts);
  }

  /// 获取指定命名空间下的运行时翻译文本，未找到时返回 "namespace.key"
  pub fn get_runtime_text(&self, namespace: &str, key: &str) -> String {
    self
      .runtime_texts
      .get(namespace)
      .and_then(|texts| texts.get(key))
      .cloned()
      .unwrap_or_else(|| format!("{}.{}", namespace, key))
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
