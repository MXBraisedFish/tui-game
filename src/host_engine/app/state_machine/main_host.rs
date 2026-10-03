//! Selection between the program page tree and the active game runtime.

use super::{GameState, HostState};

/// The currently active program-page or game-runtime branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MainHostState {
  /// The operation is host.
  Host(HostState),
  /// The operation is game.
  Game(GameState),
}

impl MainHostState {
  /// Report whether this main host state is host.
  pub fn is_host(&self) -> bool {
    matches!(self, MainHostState::Host(_))
  }

  /// Report whether this main host state is game.
  pub fn is_game(&self) -> bool {
    matches!(self, MainHostState::Game(_))
  }

  /// Return the current host.
  pub fn host(&self) -> Option<&HostState> {
    match self {
      MainHostState::Host(host) => Some(host),
      _ => None,
    }
  }

  /// Return mutable access to the owned host.
  pub fn host_mut(&mut self) -> Option<&mut HostState> {
    match self {
      MainHostState::Host(host) => Some(host),
      _ => None,
    }
  }

  /// Return the current game.
  pub fn game(&self) -> Option<&GameState> {
    match self {
      MainHostState::Game(game) => Some(game),
      _ => None,
    }
  }
}
