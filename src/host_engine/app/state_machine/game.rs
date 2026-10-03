//! Application game state and ownership of its runtime view.

use super::HostState;
use crate::host_engine::core::PackageId;

/// The retained state of game.
///
/// # Fields
///
/// * `package` - The validated package snapshot.
/// * `min_width` - The min width in terminal columns.
/// * `min_height` - The min height in terminal rows.
/// * `target_fps` - The requested frame rate, or no explicit limit.
/// * `return_host` - The return host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
  /// The validated package snapshot.
  pub package: PackageId,
  /// The min width in terminal columns.
  pub min_width: u32,
  /// The min height in terminal rows.
  pub min_height: u32,
  /// The requested frame rate, or no explicit limit.
  pub target_fps: Option<u32>,
  /// The return host.
  pub return_host: Box<HostState>,
}

impl GameState {
  /// Create a game state initialized from `package`, `min_width`, `min_height`, `target_fps`,
  /// `return_host`.
  ///
  /// # Arguments
  ///
  /// * `package` - The validated package snapshot.
  /// * `min_width` - The min width in terminal columns.
  /// * `min_height` - The min height in terminal rows.
  /// * `target_fps` - The requested frame rate, or no explicit limit.
  /// * `return_host` - The return host.
  pub fn new(
    package: PackageId,
    min_width: u32,
    min_height: u32,
    target_fps: Option<u32>,
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
