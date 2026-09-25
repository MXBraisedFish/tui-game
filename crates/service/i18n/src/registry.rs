use std::fs;

use serde::Deserialize;

use crate::host_engine::services::{LogService, LogSource, StorageService};

/// 语言注册表条目
#[derive(Clone, Debug, Deserialize)]
pub struct LanguageRegistryEntry {
  pub code: String,
  pub name: String,
  pub direction: String,
}

/// 从磁盘加载语言注册表
pub fn load_language_registry(
  storage: &StorageService,
  log: &mut LogService,
) -> Vec<LanguageRegistryEntry> {
  let path = storage.language_registry_path();

  let content = match fs::read_to_string(&path) {
    Ok(content) => content,
    Err(error) => {
      log.warn_operation_failed(
        LogSource::I18n,
        "read_language_registry",
        path.display().to_string(),
        error.to_string(),
      );
      return Vec::new();
    }
  };

  match serde_json::from_str::<Vec<LanguageRegistryEntry>>(&content) {
    Ok(registry) => registry,
    Err(error) => {
      log.warn_operation_failed(
        LogSource::I18n,
        "parse_language_registry",
        path.display().to_string(),
        error.to_string(),
      );
      Vec::new()
    }
  }
}
