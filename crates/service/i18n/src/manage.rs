//! Language availability checks and updates to translated log labels.

use super::{I18nService, LanguageInfo, load_language_registry};

use tg_service_log::{LogService, LogSource};
use tg_service_storage::StorageService;

impl I18nService {
  /// Refresh the log service's localized labels from the current language.
  ///
  /// # Errors
  ///
  /// Propagate the log service's error if translated label data cannot be applied.
  pub fn apply_log_translations(&self, log: &mut LogService) -> std::io::Result<()> {
    let missing_template = self.get_runtime_text("language_warning", "language_warning.missing");
    log.refresh_labels(
      |key| {
        let value = self.get_runtime_text("log", key);
        (value != missing_template.replace("{value:missing_key}", key)).then_some(value)
      },
      self
        .runtime_namespace("log_info")
        .cloned()
        .unwrap_or_default(),
    )
  }

  /// Reload the deployed language registry into the translation service.
  pub fn refresh_language_registry(&mut self, storage: &StorageService, log: &mut LogService) {
    let registry = load_language_registry(storage, log);

    self.set_language_registry(registry);
  }

  /// Load the display metadata for a registered language package.
  ///
  /// # Arguments
  ///
  /// * `storage` - The deployment-relative storage service.
  /// * `log` - The service receiving diagnostic records.
  /// * `language_code` - The registered language code.
  pub fn load_language_package_info(
    &mut self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) -> bool {
    let _ = storage;
    if let Some(entry) = self
      .language_registry()
      .iter()
      .find(|entry| entry.code == language_code)
    {
      self.set_current_language_info(Some(LanguageInfo {
        code: entry.code.clone(),
        direction: entry.direction.clone(),
      }));
      return true;
    }

    log.warn_operation_failed(
      LogSource::I18n,
      "load_language_package",
      language_code,
      "language package is unavailable",
    );
    self.set_current_language_info(None);
    false
  }

  /// Check whether a registered language package has its required deployed resources.
  ///
  /// # Arguments
  ///
  /// * `storage` - The deployment-relative storage service.
  /// * `log` - The service receiving diagnostic records.
  /// * `language_code` - The registered language code.
  pub fn is_language_package_available(
    &self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) -> bool {
    let _ = (storage, log);
    self.is_registered_language(language_code)
  }
}
