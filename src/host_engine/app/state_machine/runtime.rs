//! Application runtime updates, input ownership, service events, script callbacks, and frame presentation.

use super::{HostState, MainHostState, OverlayStackState};

/// The exception exit countdown seconds used by this module.
pub const EXCEPTION_EXIT_COUNTDOWN_SECONDS: u8 = 3;

/// The normal or exceptional shutdown request retained by the runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeClosingState {
  /// The operation is requested.
  Requested,
  /// The operation is export warning.
  ExportWarning,
  /// The operation is waiting for exports.
  WaitingForExports,

  /// The operation is stopping.
  Stopping {
    /// The waiting for exports.
    waiting_for_exports: bool,
  },
  /// The operation is exception.
  Exception {
    /// The seconds left.
    seconds_left: u8,
  },
}

/// The retained state of runtime.
///
/// # Fields
///
/// * `main_host` - The main host.
/// * `overlays` - The overlays.
/// * `closing` - The closing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeState {
  /// The main host.
  pub main_host: MainHostState,
  /// The overlays.
  pub overlays: OverlayStackState,
  /// The closing.
  pub closing: Option<RuntimeClosingState>,
}

impl RuntimeState {
  /// Create a runtime containing the program page tree and an empty overlay stack.
  pub fn new_host_runtime() -> Self {
    Self {
      main_host: MainHostState::Host(HostState::new()),
      overlays: OverlayStackState::new(),
      closing: None,
    }
  }

  /// Return the current main host.
  pub fn main_host(&self) -> &MainHostState {
    &self.main_host
  }

  /// Return mutable access to the owned main host.
  pub fn main_host_mut(&mut self) -> &mut MainHostState {
    &mut self.main_host
  }

  /// Return the current overlays.
  pub fn overlays(&self) -> &OverlayStackState {
    &self.overlays
  }

  /// Return mutable access to the owned overlays.
  pub fn overlays_mut(&mut self) -> &mut OverlayStackState {
    &mut self.overlays
  }

  /// Update the main host used by this runtime state.
  pub fn set_main_host(&mut self, main_host: MainHostState) {
    self.main_host = main_host;
  }

  /// Return the current closing.
  pub fn closing(&self) -> Option<RuntimeClosingState> {
    self.closing
  }

  /// Record a normal runtime close request.
  pub fn request_close(&mut self) {
    self.closing = Some(RuntimeClosingState::Requested);
  }

  /// Update the closing used by this runtime state.
  pub fn set_closing(&mut self, state: RuntimeClosingState) {
    self.closing = Some(state);
  }

  /// Record an exceptional runtime close request.
  pub fn request_exception_close(&mut self) {
    self.closing = Some(RuntimeClosingState::Exception {
      seconds_left: EXCEPTION_EXIT_COUNTDOWN_SECONDS,
    });
  }

  /// Clear the pending runtime close request.
  pub fn cancel_close(&mut self) {
    self.closing = None;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn shutdown_request_keeps_runtime_alive_until_preparation_finishes() {
    let mut state = RuntimeState::new_host_runtime();
    state.request_close();
    assert_eq!(state.closing(), Some(RuntimeClosingState::Requested));

    state.set_closing(RuntimeClosingState::ExportWarning);
    assert_eq!(state.closing(), Some(RuntimeClosingState::ExportWarning));

    state.cancel_close();
    assert_eq!(state.closing(), None);
  }

  #[test]
  fn exception_shutdown_always_starts_with_three_seconds() {
    let mut state = RuntimeState::new_host_runtime();
    state.request_exception_close();
    assert_eq!(
      state.closing(),
      Some(RuntimeClosingState::Exception { seconds_left: 3 })
    );
  }
}
