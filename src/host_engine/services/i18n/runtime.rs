use std::collections::HashMap;
use std::fs;

use super::embedded;
use super::service::I18nService;
use crate::host_engine::services::{LogService, LogSource, StorageService};

// 语言命名空间白名单：文件名即 namespace，文件内键值由用户自由命名
const RUNTIME_NAMESPACES: &[&str] = &["home", "settings", "terminal", "language", "mods"];

impl I18nService {
  /// 加载运行时语言包。
  /// 目标语言失败 → 回退默认语言（磁盘）→ 再失败 → 嵌入 en_us 最终保底。
  pub fn load_runtime_language(
    &mut self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) {
    self.clear_runtime_texts();

    // 1. 尝试目标语言（磁盘）
    if self.load_namespaces_for(storage, log, language_code) {
      self.set_current_language(language_code);
      return;
    }

    // 2. 回退默认语言（磁盘）
    let fallback = storage.default_language_code();
    if language_code != fallback {
      if self.load_namespaces_for(storage, log, fallback) {
        self.set_current_language(fallback);
        return;
      }
    }

    // 3. 编译时嵌入的 en_us 内容作为最终保底
    log.warn(
      LogSource::I18n,
      "All disk language loads failed, falling back to embedded en_us".to_string(),
    );
    self.load_embedded_fallback();
    self.set_current_language(fallback);
  }

  /// 从编译时嵌入的数据加载（最终保底，不依赖任何外部文件）。
  pub fn load_embedded_fallback(&mut self) {
    self.clear_runtime_texts();
    for namespace in RUNTIME_NAMESPACES {
      let mut map = HashMap::new();
      if embedded::fill_embedded_namespace(namespace, &mut map) {
        self.insert_runtime_namespace(*namespace, map);
      }
    }
  }

  /// 遍历白名单，读取 runtime/ 下的 JSON 文件，存入对应 namespace
  fn load_namespaces_for(
    &mut self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) -> bool {
    let mut loaded_any = false;

    for namespace in RUNTIME_NAMESPACES {
      let path = storage.language_runtime_namespace_path(language_code, namespace);

      let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
          log.warn(
            LogSource::I18n,
            format!("Failed to read {}: {}", path.display(), e),
          );
          continue;
        }
      };

      let texts = match serde_json::from_str::<HashMap<String, String>>(&content) {
        Ok(t) => t,
        Err(e) => {
          log.warn(
            LogSource::I18n,
            format!("Failed to parse {}: {}", path.display(), e),
          );
          continue;
        }
      };

      self.insert_runtime_namespace(*namespace, texts);
      loaded_any = true;
    }

    loaded_any
  }
}
