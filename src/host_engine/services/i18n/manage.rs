use super::{I18nService, load_language_info, load_language_registry};

use crate::host_engine::services::{LogService, LogSource, StorageService};

impl I18nService {

  /// 刷新语言注册表
  pub fn refresh_language_registry(&mut self, storage: &StorageService, log: &mut LogService) {
    let registry = load_language_registry(storage, log);

    log.info(
      LogSource::I18n,
      format!("Loaded {} language registry entries.", registry.len()),
    );

    self.set_language_registry(registry);
  }

  /// 加载语言包信息
  pub fn load_language_package_info(
    &mut self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) -> bool {
    let info = load_language_info(storage, log, language_code);

    if info.is_some() {
      self.set_current_language_info(info);
      return true;
    }

    self.set_current_language_info(None);
    false
  }

  /// 检查语言包是否可用
  pub fn is_language_package_available(
    &self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) -> bool {
    load_language_info(storage, log, language_code).is_some()
  }
}
