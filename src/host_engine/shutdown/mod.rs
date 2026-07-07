use crate::host_engine::core::{ExitState, RuntimeWorld};
use crate::host_engine::services::EngineServices;

use super::services::LogSource;

/// 执行引擎关闭流程：记录日志并退出终端
pub fn close(services: &mut EngineServices, _world: RuntimeWorld, _exit_state: ExitState) {
  let _ = services.input_method.release_input_method();

  services
    .log
    .info(LogSource::Shutdown, "[Shutdown] Engine closed.");

  services.terminal.exit();
}
