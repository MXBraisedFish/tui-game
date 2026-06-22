#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayStackState {
  pub stack: Vec<OverlayState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayState {
  pub kind: OverlayKind,
  pub logic: OverlayLogicState,
  pub render: OverlayRenderState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayKind {
  ConfirmExit,
  WindowSizeWarning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayLogicState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayRenderState {
  pub required_width: u32,
  pub required_height: u32,
}

// 覆盖层状态
impl OverlayStackState {
  // 创建空的覆盖层栈
  pub fn new() -> Self {
    Self { stack: Vec::new() }
  }

  // 覆盖层是否为空
  pub fn is_empty(&self) -> bool {
    self.stack.is_empty()
  }

  // 覆盖层可用数量
  pub fn len(&self) -> usize {
    self.stack.len()
  }

  // 获取顶层覆盖层
  pub fn top(&self) -> Option<&OverlayState> {
    self.stack.last()
  }

  // 获取顶层覆盖层（可变）
  pub fn top_mut(&mut self) -> Option<&mut OverlayState> {
    self.stack.last_mut()
  }

  // 添加覆盖层
  pub fn push(&mut self, overlay: OverlayState) {
    self.stack.push(overlay);
  }

  // 弹出覆盖层
  pub fn pop(&mut self) -> Option<OverlayState> {
    self.stack.pop()
  }

  // 当前覆盖层类型
  pub fn current_kind(&self) -> Option<OverlayKind> {
    self.top().map(|o| o.kind)
  }

  // 清理覆盖层
  pub fn clear(&mut self) {
    self.stack.clear();
  }
}
