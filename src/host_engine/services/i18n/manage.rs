use super::{I18nService, load_language_info, load_language_registry};

use crate::host_engine::services::{LogService, LogSource, StorageService};

impl I18nService {
  // 加载语言注册表
  pub fn refresh_language_registry(&mut self, storage: &StorageService, log: &mut LogService) {
    // 读取内容
    let registry = load_language_registry(storage, log);

    // 加载成功
    // TODO:之后删掉
    log.info(
      LogSource::I18n,
      format!("Loaded {} language registry entries.", registry.len()),
    );

    // 设置语言注册表
    self.set_language_registry(registry);
  }

  // 检查并加载语言信息
  pub fn load_language_package_info(
    &mut self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) -> bool {
    // 获取信息
    let info = load_language_info(storage, log, language_code);

    // 检查是否有内容
    if info.is_some() {
      // 设置语言信息,并返回成功
      self.set_current_language_info(info);
      return true;
    }

    // 失败
    self.set_current_language_info(None);
    false
  }

  // 判断语言包是否可用
  pub fn is_language_package_available(
    &self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) -> bool {
    load_language_info(storage, log, language_code).is_some()
  }
}
