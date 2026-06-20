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
  Home,
  Settings,
  LanguageSelect,
  Mods,
  TerminalCheck,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeLogicState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeRenderState;

// UI节点状态
impl UiNodeState {
  pub fn root() -> Self {
    Self {
      kind: UiNodeKind::Root,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  pub fn home() -> Self {
    Self {
      kind: UiNodeKind::Home,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  pub fn settings() -> Self {
    Self {
      kind: UiNodeKind::Settings,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  pub fn mods() -> Self {
    Self {
      kind: UiNodeKind::Mods,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  pub fn terminal_check() -> Self {
    Self {
      kind: UiNodeKind::TerminalCheck,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  pub fn language_select() -> Self {
    Self {
      kind: UiNodeKind::LanguageSelect,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }
}

// UI树状态
impl UiTreeState {
  // 创建以根节点初始化的 UI 树
  pub fn new() -> Self {
    Self {
      path: vec![UiNodeState::home()],
    }
  }

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

  // 进入新节点
  pub fn enter(&mut self, node: UiNodeState) {
    self.path.push(node);
  }

  // 返回上一级节点
  pub fn back(&mut self) -> Option<UiNodeState> {
    if self.path.len() <= 1 {
      return None;
    }

    self.path.pop()
  }

  // 重置UI树
  pub fn reset(&mut self, root: UiNodeState) {
    self.path.clear();
    self.path.push(root);
  }

  // 替换当前节点
  pub fn replace_current(&mut self, node: UiNodeState) {
    if let Some(current) = self.path.last_mut() {
      *current = node;
    }
  }
}
