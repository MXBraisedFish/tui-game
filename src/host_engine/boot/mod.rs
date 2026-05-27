// 引用结构体
use crate::host_engine::core::{
  BootOutput,
  RuntimeWorld
};
use crate::host_engine::services::EngineServices;

// 启动函数
pub fn prepare() -> BootOutput {

  println!("[Boot] Preparing engine...");
  
  let mut services = EngineServices::new();

  println!("[Boot] Scanning packages...");

  let root_dir = services.storage.root_dir().clone();
  services.package.scan_all(&root_dir);

  println!(
    "[Boot] Found {} packages ({} games, {} screensavers, {} bosses)",
    services.package.total_count(),
    services.package.games().len(),
    services.package.screensavers().len(),
    services.package.bosses().len(),
  );

  let world = RuntimeWorld::new();

  println!(
    "[Boot] Storage root: {}",
    services.storage.root_dir().display()
  );

  // 返回启动输出
  BootOutput {
    services,
    world
  }
}