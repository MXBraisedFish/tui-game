// 引用结构体
use crate::host_engine::core::{
  RuntimeWorld,
  ExitState
};
use crate::host_engine::services::EngineServices;

// 关闭函数
pub fn close(_services: EngineServices, _world: RuntimeWorld, _exit_state: ExitState) {
  println!("[Shutdown] Engine closed.");
}