//! Application state retained across lifecycle phases.

use super::HostMachineState;
use crate::host_engine::core::{EngineClock, PackageId};

/// Application state retained while lifecycle phases initialize, run, and close the host.
///
/// # Fields
///
/// * `clock` - The clock.
/// * `state` - The host machine state carried by this runtime world.
/// * `pending_new_game` - The pending new game.
pub struct RuntimeWorld {
  /// The clock.
  pub clock: EngineClock,
  /// The host machine state carried by this runtime world.
  pub state: HostMachineState,
  /// The pending new game.
  pub pending_new_game: Option<PackageId>,
}

impl RuntimeWorld {
  /// Create a runtime world with its initial state.
  pub fn new() -> Self {
    Self {
      clock: EngineClock::new(),
      state: HostMachineState::new(),
      pending_new_game: None,
    }
  }

  /// Report whether this runtime world is stopped.
  pub fn is_stopped(&self) -> bool {
    self.state.is_stopped()
  }
}
