// 引用结构体
use crate::host_engine::core::{
  BootOutput,
  RuntimeWorld
};
use crate::host_engine::services::EngineServices;

// 启动函数
pub fn prepare() -> BootOutput {
  let mut services = EngineServices::new();

  services.log.info("[Boot] Preparing engine...");

  services.log.info("[Boot] Scanning packages...");

  let root_dir = services.storage.root_dir().clone();
  services.package.scan_all(&root_dir);

  services.log.info(
    "[Boot] Found {} packages ({} games, {} screensavers, {} bosses)"
  );

  let world = RuntimeWorld::new();

  services.log.info(
    "[Boot] Storage root: {}"
  );

  // 返回启动输出
  BootOutput {
    services,
    world
  }
}