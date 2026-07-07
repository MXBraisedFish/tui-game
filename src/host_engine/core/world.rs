use std::collections::HashSet;

use super::{EngineClock, HostMachineState};

/// 运行时世界，持有引擎时钟与主机状态机
pub struct RuntimeWorld {
  pub clock: EngineClock,
  pub state: HostMachineState,
  pub temporary_safe_mode_disabled: HashSet<String>,
}

impl RuntimeWorld {
  pub fn new() -> Self {
    Self {
      clock: EngineClock::new(),
      state: HostMachineState::new(),
      temporary_safe_mode_disabled: HashSet::new(),
    }
  }

  pub fn is_stopped(&self) -> bool {
    self.state.is_stopped()
  }
}
