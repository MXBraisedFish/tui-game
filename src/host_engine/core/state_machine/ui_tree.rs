#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiTreeState {
    pub path: Vec<UiNodeState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeState {
    pub kind: UiNodeKind,
    pub logic: UiNodeLogicState,
    pub render: UiNodeRenderState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiNodeKind {
    Root,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeLogicState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeRenderState;

// UI树状态
impl UiTreeState {
    // UI树节点
    pub fn path(&self) -> &[UiNodeState] {
        &self.path
    }

    // 当前节点
    pub fn current(&self) -> Option<&UiNodeState> {
        self.path.last()
    }

    // 当前结点（可变）
    pub fn current_mut(&mut self) -> Option<&mut UiNodeState> {
        self.path.last_mut()
    }
}
