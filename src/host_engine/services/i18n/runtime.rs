use std::collections::{HashMap, HashSet};
use std::fs;

use super::embedded;
use super::service::I18nService;
use crate::host_engine::services::{LogService, LogSource, StorageService};

const RUNTIME_NAMESPACES: &[&str] = &[
  "home",
  "host_key",
  "settings",
  "display_settings",
  "terminal",
  "language",
  "mods",
  "window_size",
  "language_warning",
  "language_loading",
  "export_loading",
  "safe_mode_warning",
  "screenshot",
  "storage_management",
  "storage_management_view",
  "storage_management_clear",
  "storage_management_export",
  "clear_warning",
  "log",
  "export_settings",
  "security_settings",
  "security_details",
  "game_pack",
  "game_list",
  "screensaver_list",
  "screensaver_pack",
  "toolbar",
];

impl I18nService {
  /// 加载运行时语言文本，含磁盘加载失败时的回退逻辑
  pub fn load_runtime_language(
    &mut self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) {
    self.clear_runtime_texts();

    let loaded = self.load_namespaces_for(storage, log, language_code);
    if !loaded.is_empty() {
      self.fill_missing_namespaces(storage, log, &loaded);
      self.set_current_language(language_code);
      return;
    }

    let fallback = storage.default_language_code();
    if language_code != fallback {
      let loaded = self.load_namespaces_for(storage, log, fallback);
      if !loaded.is_empty() {
        self.fill_missing_namespaces(storage, log, &loaded);
        self.set_current_language(fallback);
        return;
      }
    }

    log.warn(
      LogSource::I18n,
      "All disk language loads failed, falling back to embedded en_us".to_string(),
    );
    self.load_embedded_fallback();

    if self.is_runtime_empty() {
      log.error(
        LogSource::I18n,
        format!("Embedded language fallback for '{}' is empty!", fallback),
      );
    }

    self.set_current_language(fallback);
  }

  /// 加载编译时嵌入的英文回退翻译
  pub fn load_embedded_fallback(&mut self) {
    self.clear_runtime_texts();
    for namespace in RUNTIME_NAMESPACES {
      let mut map = HashMap::new();
      if embedded::fill_embedded_namespace(namespace, &mut map) {
        self.insert_runtime_namespace(*namespace, map);
      }
    }
  }

  fn load_namespaces_for(
    &mut self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) -> HashSet<&'static str> {
    let mut loaded = HashSet::new();

    for &namespace in RUNTIME_NAMESPACES {
      if let Some(texts) = Self::load_namespace_file(storage, log, language_code, namespace) {
        self.insert_runtime_namespace(namespace, texts);
        loaded.insert(namespace);
      }
    }

    loaded
  }

  fn fill_missing_namespaces(
    &mut self,
    storage: &StorageService,
    log: &mut LogService,
    loaded: &HashSet<&'static str>,
  ) {
    let fallback = storage.default_language_code();

    for &namespace in RUNTIME_NAMESPACES {
      if loaded.contains(namespace) {
        continue;
      }

      if let Some(texts) = Self::load_namespace_file(storage, log, fallback, namespace) {
        self.insert_runtime_namespace(namespace, texts);
        continue;
      }

      let mut map = HashMap::new();
      if embedded::fill_embedded_namespace(namespace, &mut map) {
        self.insert_runtime_namespace(namespace, map);
      }
    }
  }

  fn load_namespace_file(
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
    namespace: &str,
  ) -> Option<HashMap<String, String>> {
    let path = storage.language_runtime_namespace_path(language_code, namespace);

    let content = match fs::read_to_string(&path) {
      Ok(c) => c,
      Err(e) => {
        log.warn(
          LogSource::I18n,
          format!("Failed to read {}: {}", path.display(), e),
        );
        return None;
      }
    };

    match serde_json::from_str::<HashMap<String, String>>(&content) {
      Ok(t) => Some(t),
      Err(e) => {
        log.warn(
          LogSource::I18n,
          format!("Failed to parse {}: {}", path.display(), e),
        );
        None
      }
    }
  }
}
