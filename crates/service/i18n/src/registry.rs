use std::fs;

use serde::Deserialize;

use tg_service_log::{LogService, LogSource};
use tg_service_storage::StorageService;

/// An entry of the language registry.
#[derive(Clone, Debug, Deserialize)]
pub struct LanguageRegistryEntry {
  pub code: String,
  pub name: String,
  pub direction: String,
}

/// Loads the language registry from disk.
///
/// Logs a warning and returns an empty list when the file cannot be read or parsed.
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
