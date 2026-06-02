// 引用结构体
use crate::host_engine::core::{BootOutput, RuntimeWorld};
use crate::host_engine::services::EngineServices;

// 临时日志
use super::services::{LogEntry, LogLevel, LogService, LogSource, format_log_entry};

// 启动函数
pub fn prepare() -> BootOutput {
  let mut services = EngineServices::new();

  services
    .log
    .info(LogSource::Boot, "[Boot] Preparing engine...");

  services
    .log
    .info(LogSource::Boot, "[Boot] Scanning packages...");

  // 临时语言测试
  services
    .i18n
    .refresh_language_registry(&services.storage, &mut services.log);

  let preferred_language = services
    .storage
    .read_language_code()
    .unwrap_or_else(|| services.storage.default_language_code().to_string());

  let selected_language = if services.i18n.is_language_package_available(
    &services.storage,
    &mut services.log,
    &preferred_language,
  ) {
    preferred_language
  } else {
    services.storage.default_language_code().to_string()
  };

  services.i18n.load_language_package_info(
    &services.storage,
    &mut services.log,
    &selected_language,
  );

  services
    .i18n
    .load_runtime_language(&services.storage, &mut services.log, &selected_language);
  // 临时语言测试

  let root_dir = services.storage.root_dir();
  services.package.scan_all(&root_dir);

  services.log.info(
    LogSource::Boot,
    "[Boot] Found {} packages ({} games, {} screensavers, {} bosses)",
  );

  let world = RuntimeWorld::new();

  services
    .log
    .info(LogSource::Boot, "[Boot] Storage root: {}");

  // 返回启动输出
  BootOutput { services, world }
}
