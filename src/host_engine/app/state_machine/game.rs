use super::HostState;
use crate::host_engine::core::PackageId;

/// 游戏状态仅保存宿主状态机需要的身份与返回点；Lua VM 由 GameService 持有。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
  pub package: PackageId,
  pub min_width: u32,
  pub min_height: u32,
  pub target_fps: u32,
  pub return_host: Box<HostState>,
}

impl GameState {
  pub fn new(
    package: PackageId,
    min_width: u32,
    min_height: u32,
    target_fps: u32,
    return_host: HostState,
  ) -> Self {
    Self {
      package,
      min_width,
      min_height,
      target_fps,
      return_host: Box::new(return_host),
    }
  }
}
