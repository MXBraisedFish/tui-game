use crate::host_engine::core::RuntimeWorld;
use crate::host_engine::services::EngineServices;

/// 引擎启动阶段的输出，包含初始化的服务和世界
pub struct BootOutput {
  pub services: EngineServices,
  pub world: RuntimeWorld,
}
