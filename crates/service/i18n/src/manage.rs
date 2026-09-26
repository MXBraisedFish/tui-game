use super::{I18nService, LanguageInfo, load_language_registry};

use tg_service_log::{LogService, LogSource};
use tg_service_storage::StorageService;

impl I18nService {
  /// Applies the current language's log labels and `log_info` message templates to the log service.
  ///
  /// # Errors
  ///
  /// Returns an error when the pending log messages cannot be written to the log file.
  pub fn apply_log_translations(&self, log: &mut LogService) -> std::io::Result<()> {
    let missing_template = self.get_runtime_text("language_warning", "language_warning.missing");
    log.refresh_labels(
      |key| {
        let value = self.get_runtime_text("log", key);
        (value != missing_template.replace("{value:missing_key}", key)).then_some(value)
      },
      self.runtime_namespace("log_info").cloned().unwrap_or_default(),
    )
  }

  /// Reloads the language registry from disk.
  pub fn refresh_language_registry(&mut self, storage: &StorageService, log: &mut LogService) {
    let registry = load_language_registry(storage, log);

    self.set_language_registry(registry);
  }

  /// Loads the language package info of `language_code` from the registry.
  ///
  /// Returns `false` and clears the current language info when the language is not registered.
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

  /// Returns whether the language package of `language_code` is available.
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
