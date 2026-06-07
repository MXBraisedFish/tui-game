use super::UiTreeState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostState {
    pub ui_tree: UiTreeState,
}

// 宿主状态
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
