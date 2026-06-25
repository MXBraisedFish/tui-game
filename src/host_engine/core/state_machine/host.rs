use super::UiTreeState;

/// Host 界面状态，持有 UI 树
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostState {
  pub ui_tree: UiTreeState,
}

impl HostState {
  pub fn new() -> Self {
    Self {
      ui_tree: UiTreeState::new(),
    }
  }

  pub fn ui_tree(&self) -> &UiTreeState {
    &self.ui_tree
  }

  pub fn ui_tree_mut(&mut self) -> &mut UiTreeState {
    &mut self.ui_tree
  }
}
