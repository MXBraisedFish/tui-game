// 引用结构体
use crate::host_engine::core::{
  RuntimeWorld,
  ExitState
};
use crate::host_engine::services::EngineServices;

// 关闭函数
pub fn close(services: &mut EngineServices, world: RuntimeWorld, exit_state: ExitState) {
  services.log.info("[Shutdown] Engine closed.");

  // 退出终端模式
  services.terminal.exit();
}