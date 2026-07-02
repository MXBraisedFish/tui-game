/// 覆盖层栈状态，以栈形式管理多个覆盖层
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayStackState {
  pub stack: Vec<OverlayState>,
}

/// 覆盖层状态，包含类型及其逻辑与渲染状态
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayState {
  pub kind: OverlayKind,
  pub logic: OverlayLogicState,
  pub render: OverlayRenderState,
}

/// 覆盖层类型枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayKind {
  ConfirmExit,
  LanguageLoading,
  WindowSizeWarning,
}

/// 覆盖层逻辑状态
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayLogicState;

/// 覆盖层渲染状态，包含该覆盖层所需的最小窗口尺寸
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayRenderState {
  pub required_width: u32,
  pub required_height: u32,
}

impl OverlayStackState {
  pub fn new() -> Self {
    Self { stack: Vec::new() }
  }

  pub fn is_empty(&self) -> bool {
    self.stack.is_empty()
  }

  pub fn len(&self) -> usize {
    self.stack.len()
  }

  pub fn top(&self) -> Option<&OverlayState> {
    self.stack.last()
  }

  pub fn top_mut(&mut self) -> Option<&mut OverlayState> {
    self.stack.last_mut()
  }

  /// 压入一个覆盖层到栈顶
  pub fn push(&mut self, overlay: OverlayState) {
    self.stack.retain(|item| item.kind != overlay.kind);
    self.stack.push(overlay);
    self.stack.sort_by_key(|item| item.kind.priority());
  }

  /// 弹出栈顶覆盖层
  pub fn pop(&mut self) -> Option<OverlayState> {
    self.stack.pop()
  }

  pub fn current_kind(&self) -> Option<OverlayKind> {
    self.top().map(|o| o.kind)
  }

  pub fn remove_kind(&mut self, kind: OverlayKind) -> Option<OverlayState> {
    let index = self.stack.iter().position(|overlay| overlay.kind == kind)?;
    Some(self.stack.remove(index))
  }

  /// 清空所有覆盖层
  pub fn clear(&mut self) {
    self.stack.clear();
  }
}

impl OverlayKind {
  fn priority(self) -> u8 {
    match self {
      OverlayKind::ConfirmExit => 10,
      OverlayKind::LanguageLoading => 20,
      OverlayKind::WindowSizeWarning => 30,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn overlay(kind: OverlayKind) -> OverlayState {
    OverlayState {
      kind,
      logic: OverlayLogicState,
      render: OverlayRenderState {
        required_width: 0,
        required_height: 0,
      },
    }
  }

  #[test]
  fn highest_priority_overlay_is_current() {
    let mut stack = OverlayStackState::new();
    stack.push(overlay(OverlayKind::LanguageLoading));
    stack.push(overlay(OverlayKind::WindowSizeWarning));

    assert_eq!(stack.current_kind(), Some(OverlayKind::WindowSizeWarning));

    stack.remove_kind(OverlayKind::WindowSizeWarning);
    assert_eq!(stack.current_kind(), Some(OverlayKind::LanguageLoading));
  }

  #[test]
  fn pushing_same_overlay_kind_replaces_old_state() {
    let mut stack = OverlayStackState::new();
    stack.push(overlay(OverlayKind::LanguageLoading));
    stack.push(overlay(OverlayKind::LanguageLoading));

    assert_eq!(stack.len(), 1);
    assert_eq!(stack.current_kind(), Some(OverlayKind::LanguageLoading));
  }
}
