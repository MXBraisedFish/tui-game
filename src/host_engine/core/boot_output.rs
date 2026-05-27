// 引用结构体
use crate::host_engine::core::RuntimeWorld;
use crate::host_engine::services::EngineServices;

// 启动输出
// 启动阶段结束后的输出内容结构体
pub struct BootOutput {
  pub services: EngineServices,
  pub world: RuntimeWorld,
}