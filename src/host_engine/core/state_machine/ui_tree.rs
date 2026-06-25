
/// UI 树状态，以栈形式管理界面节点的导航路径
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiTreeState {
  pub path: Vec<UiNodeState>,
}

/// UI 节点状态，包含节点类型及其逻辑与渲染状态
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeState {
  pub kind: UiNodeKind,
  pub logic: UiNodeLogicState,
  pub render: UiNodeRenderState,
}

/// UI 节点类型枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiNodeKind {
  Root,
  Home,
  Settings,
  LanguageSelect,
  Mods,
  TerminalCheck,
  InputDemo,
}

/// UI 节点逻辑状态
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeLogicState;

/// UI 节点渲染状态
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeRenderState;

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

  pub fn input_demo() -> Self {
    Self {
      kind: UiNodeKind::InputDemo,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }
}

impl UiTreeState {
  pub fn new() -> Self {
    Self {
      path: vec![UiNodeState::home()],
    }
  }

  pub fn path(&self) -> &[UiNodeState] {
    &self.path
  }

  pub fn current(&self) -> Option<&UiNodeState> {
    self.path.last()
  }

  pub fn current_mut(&mut self) -> Option<&mut UiNodeState> {
    self.path.last_mut()
  }

  /// 进入一个 UI 节点，将其压入导航栈
  pub fn enter(&mut self, node: UiNodeState) {
    self.path.push(node);
  }

  /// 返回上一层 UI 节点，仅在栈深度大于 1 时执行
  pub fn back(&mut self) -> Option<UiNodeState> {
    if self.path.len() <= 1 {
      return None;
    }

    self.path.pop()
  }

  /// 重置 UI 树，以指定根节点替换整个导航栈
  pub fn reset(&mut self, root: UiNodeState) {
    self.path.clear();
    self.path.push(root);
  }

  /// 替换当前栈顶 UI 节点
  pub fn replace_current(&mut self, node: UiNodeState) {
    if let Some(current) = self.path.last_mut() {
      *current = node;
    }
  }
}
