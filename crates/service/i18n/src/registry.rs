//! Reading the deployed registry of selectable languages.

use std::fs;

use serde::Deserialize;

use tg_service_log::{LogService, LogSource};
use tg_service_storage::StorageService;

/// A registered language and the metadata required to locate its resources.
///
/// # Fields
///
/// * `code` - The stable error or language code.
/// * `name` - The name used to identify the object or field.
/// * `direction` - The direction.
#[derive(Clone, Debug, Deserialize)]
pub struct LanguageRegistryEntry {
  /// The stable error or language code.
  pub code: String,
  /// The name used to identify the object or field.
  pub name: String,
  /// The direction.
  pub direction: String,
}

/// Read registered language metadata and report malformed or missing registry data.
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
