use std::collections::HashMap;

pub struct I18nService {
  current_language: String,
  runtime_texts: HashMap<String, HashMap<String, String>>,
}

impl I18nService {
  pub fn new() -> Self {
    Self {
      current_language: String::new(),
      runtime_texts: HashMap::new(),
    }
  }

  // 当前语言
  pub fn current_language(&self) -> &str {
    &self.current_language
  }

  // 设置当前语言
  pub fn set_current_language(&mut self, language_code: impl Into<String>) {
    self.current_language = language_code.into();
  }

  // 清理运行文本
  pub fn clear_runtime_texts(&mut self) {
    self.runtime_texts.clear();
  }

  // 插入运行命名空间
  pub fn insert_runtime_namespace(&mut self, namespace: impl Into<String>, texts: HashMap<String, String>,
  ) {
    self.runtime_texts.insert(namespace.into(), texts);
  }

  // 获取运行文本
  pub fn get_runtime_text(&self, namespace: &str, key: &str) -> String {
    self.runtime_texts.get(namespace).and_then(|texts| texts.get(key)).cloned().unwrap_or_else(|| format!("{}.{}", namespace, key))
  }
}