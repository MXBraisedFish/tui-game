//! Program-owned page tree and navigation state.

use super::UiTreeState;

/// The retained state of host.
///
/// # Fields
///
/// * `ui_tree` - The UI tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostState {
  /// The UI tree.
  pub ui_tree: UiTreeState,
}

impl HostState {
  /// Create a host state with its initial state.
  pub fn new() -> Self {
    Self {
      ui_tree: UiTreeState::new(),
    }
  }

  /// Return the current UI tree.
  pub fn ui_tree(&self) -> &UiTreeState {
    &self.ui_tree
  }

  /// Return mutable access to the owned UI tree.
  pub fn ui_tree_mut(&mut self) -> &mut UiTreeState {
    &mut self.ui_tree
  }
}
