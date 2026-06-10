use std::collections::HashMap;
use std::fs;

use super::service::I18nService;
use crate::host_engine::services::{LogService, LogSource, StorageService};

// 语言命名空间白名单：文件名即 namespace，文件内键值由用户自由命名
const RUNTIME_NAMESPACES: &[&str] = &["home", "settings"];

impl I18nService {
  /// 加载运行时语言包，失败时回退到默认语言
  pub fn load_runtime_language(
    &mut self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) {
    self.clear_runtime_texts();

    // 尝试目标语言
    if self.load_namespaces_for(storage, log, language_code) {
      self.set_current_language(language_code);
      return;
    }

    // 回退到默认语言
    let fallback = storage.default_language_code();
    if language_code != fallback {
      self.load_namespaces_for(storage, log, fallback);
    }
    self.set_current_language(fallback);
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
