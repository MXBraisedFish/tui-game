use std::collections::{HashMap, HashSet};
use std::fs;

use super::embedded;
use super::service::I18nService;
use crate::host_engine::services::{LogService, LogSource, StorageService};

const RUNTIME_NAMESPACES: &[&str] = &[
  "boot_loading",
  "home",
  "host_key",
  "settings",
  "key_bindings",
  "key_bindings_global",
  "key_bindings_game",
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
  "recording",
  "storage_management",
  "storage_management_view",
  "storage_management_clear",
  "storage_management_export",
  "clear_warning",
  "cover_continue",
  "log",
  "log_info",
  "export_settings",
  "security_settings",
  "security_details",
  "game_pack",
  "game_list",
  "screensaver_list",
  "screenshot_recording",
  "screenshot_settings",
  "screenshot_list",
  "recording_list",
  "fonts_settings",
  "screensaver_pack",
  "toolbar",
  "toolbar_custom",
  "recording_settings",
  "exit_warning",
  "game_warning",
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
      self.fill_missing_runtime_texts(storage, log, language_code);
      self.set_current_language(language_code);
      return;
    }

    let fallback = storage.default_language_code();
    if language_code != fallback {
      let loaded = self.load_namespaces_for(storage, log, fallback);
      if !loaded.is_empty() {
        self.fill_missing_runtime_texts(storage, log, fallback);
        self.set_current_language(fallback);
        return;
      }
    }

    log.warn_message(
      LogSource::I18n,
      crate::host_engine::services::HostLogMessage::new(
        "log_info.fallback.activated",
        "{domain} entered fallback mode: {reason}",
      )
      .param("domain", "i18n")
      .param("reason", "disk language resources are unavailable"),
    );
    self.load_embedded_fallback();

    if self.is_runtime_empty() {
      log.error_operation_failed(
        LogSource::I18n,
        "load_embedded_language",
        fallback,
        "embedded language fallback is empty",
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

  fn fill_missing_runtime_texts(
    &mut self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) {
    let fallback = storage.default_language_code();

    for &namespace in RUNTIME_NAMESPACES {
      if language_code != fallback
        && let Some(texts) = Self::load_namespace_file(storage, log, fallback, namespace)
      {
        self.merge_runtime_namespace(namespace, texts);
      }

      let mut map = HashMap::new();
      if embedded::fill_embedded_namespace(namespace, &mut map) {
        self.merge_runtime_namespace(namespace, map);
      }
    }
  }

  fn load_namespace_file(
    storage: &StorageService,
    _log: &mut LogService,
    language_code: &str,
    namespace: &str,
  ) -> Option<HashMap<String, String>> {
    let path = storage.language_runtime_namespace_path(language_code, namespace);

    let content = match fs::read_to_string(&path) {
      Ok(c) => c,
      Err(_) => return None,
    };

    match serde_json::from_str::<HashMap<String, String>>(&content) {
      Ok(t) => Some(t),
      Err(_) => None,
    }
  }
}
