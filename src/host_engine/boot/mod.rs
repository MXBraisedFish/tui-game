// 引用结构体
use crate::host_engine::core::{
  BootOutput,
  RuntimeWorld
};
use crate::host_engine::services::EngineServices;

// 临时日志
use super::services::{LogEntry, LogLevel, LogService, LogSource, format_log_entry};

// 启动函数
pub fn prepare() -> BootOutput {
  let mut services = EngineServices::new();

  services.log.info(LogSource::Boot, "[Boot] Preparing engine...");

  services.log.info(LogSource::Boot, "[Boot] Scanning packages...");

  // 临时语言测试
  let language_code = services
    .storage
    .read_language_code()
    .unwrap_or_else(|| services.storage.default_language_code().to_string());

  services.i18n.load_runtime_test_language(
    &services.storage,
    &mut services.log,
    &language_code,
  );
  // 临时语言测试

  let root_dir = services.storage.root_dir();
  services.package.scan_all(&root_dir);

  services.log.info(LogSource::Boot, 
    "[Boot] Found {} packages ({} games, {} screensavers, {} bosses)"
  );

  let world = RuntimeWorld::new();

  services.log.info(LogSource::Boot, 
    "[Boot] Storage root: {}"
  );

  // 返回启动输出
  BootOutput {
    services,
    world
  }
}