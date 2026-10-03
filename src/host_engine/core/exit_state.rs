//! Closing status and active-write diagnostics carried into shutdown.

/// The retained state of exit.
pub struct ExitState {}

impl ExitState {
  /// Create an exit state with its initial state.
  pub fn new() -> Self {
    Self {}
  }
}
