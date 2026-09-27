use super::HostMachineState;
use crate::host_engine::core::{EngineClock, PackageId};

/// 运行时世界，持有引擎时钟与主机状态机
pub struct RuntimeWorld {
  pub clock: EngineClock,
  pub state: HostMachineState,
  pub pending_new_game: Option<PackageId>,
}

impl RuntimeWorld {
  pub fn new() -> Self {
    Self {
      clock: EngineClock::new(),
      state: HostMachineState::new(),
      pending_new_game: None,
    }
  }

  pub fn is_stopped(&self) -> bool {
    self.state.is_stopped()
  }
}
