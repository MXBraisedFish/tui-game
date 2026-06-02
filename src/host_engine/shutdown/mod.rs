// 引用结构体
use crate::host_engine::core::{ExitState, RuntimeWorld};
use crate::host_engine::services::EngineServices;

// 临时日志
use super::services::{LogEntry, LogLevel, LogService, LogSource, format_log_entry};

// 关闭函数
pub fn close(services: &mut EngineServices, world: RuntimeWorld, exit_state: ExitState) {
  services
    .log
    .info(LogSource::Shutdown, "[Shutdown] Engine closed.");

  // 退出终端模式
  services.terminal.exit();
}
