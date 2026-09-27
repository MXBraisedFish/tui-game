/// 覆盖层栈状态，以栈形式管理多个覆盖层
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayStackState {
  pub stack: Vec<OverlayState>,
  transitions: Vec<OverlayStackTransition>,
}

/// 覆盖屏栈从空到非空、或从非空回到空时产生的生命周期变化。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayStackTransition {
  Started,
  Stopped,
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
  CoverContinue,
  ClearWarning,
  ExportLoading,
  ExportSettings,
  GameWarning,
  LanguageLoading,
  ScreenshotCapture,
  Screensaver,
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
    Self {
      stack: Vec::new(),
      transitions: Vec::new(),
    }
  }

  pub fn top(&self) -> Option<&OverlayState> {
    self.stack.last()
  }

  pub fn top_mut(&mut self) -> Option<&mut OverlayState> {
    self.stack.last_mut()
  }

  fn current_index(&self) -> Option<usize> {
    self
      .stack
      .iter()
      .enumerate()
      .max_by(|(left_index, left), (right_index, right)| {
        left
          .kind
          .priority()
          .cmp(&right.kind.priority())
          .then_with(|| left_index.cmp(right_index))
      })
      .map(|(index, _)| index)
  }

  /// 压入一个覆盖层到栈顶
  pub fn push(&mut self, overlay: OverlayState) {
    let was_empty = self.stack.is_empty();
    self.stack.retain(|item| item.kind != overlay.kind);
    self.stack.push(overlay);
    self.stack.sort_by_key(|item| item.kind.priority());
    if was_empty {
      self.transitions.push(OverlayStackTransition::Started);
    }
  }

  /// 弹出栈顶覆盖层
  pub fn pop(&mut self) -> Option<OverlayState> {
    let overlay = self.stack.pop()?;
    if self.stack.is_empty() {
      self.transitions.push(OverlayStackTransition::Stopped);
    }
    Some(overlay)
  }

  pub fn current_kind(&self) -> Option<OverlayKind> {
    self.current_index().map(|index| self.stack[index].kind)
  }

  pub fn remove_kind(&mut self, kind: OverlayKind) -> Option<OverlayState> {
    let index = self.stack.iter().position(|overlay| overlay.kind == kind)?;
    let overlay = self.stack.remove(index);
    if self.stack.is_empty() {
      self.transitions.push(OverlayStackTransition::Stopped);
    }
    Some(overlay)
  }

  pub fn get(&self, kind: OverlayKind) -> Option<&OverlayState> {
    self.stack.iter().find(|overlay| overlay.kind == kind)
  }

  pub fn drain_transitions(&mut self) -> Vec<OverlayStackTransition> {
    std::mem::take(&mut self.transitions)
  }
}

impl OverlayKind {
  fn priority(self) -> u8 {
    match self {
      OverlayKind::CoverContinue => 20,
      OverlayKind::ClearWarning => 20,
      OverlayKind::ExportLoading => 20,
      OverlayKind::ExportSettings => 20,
      OverlayKind::LanguageLoading => 20,
      OverlayKind::Screensaver => 25,
      OverlayKind::GameWarning => 27,
      OverlayKind::WindowSizeWarning => 30,
      OverlayKind::ScreenshotCapture => 40,
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
  fn screenshot_capture_overrides_window_size_warning() {
    let mut stack = OverlayStackState::new();
    stack.push(overlay(OverlayKind::WindowSizeWarning));
    stack.push(overlay(OverlayKind::ScreenshotCapture));

    assert_eq!(stack.current_kind(), Some(OverlayKind::ScreenshotCapture));
  }

  #[test]
  fn screenshot_then_window_then_screensaver_then_regular_overlay() {
    let mut stack = OverlayStackState::new();
    stack.push(overlay(OverlayKind::LanguageLoading));
    stack.push(overlay(OverlayKind::Screensaver));
    stack.push(overlay(OverlayKind::WindowSizeWarning));
    stack.push(overlay(OverlayKind::ScreenshotCapture));

    assert_eq!(stack.current_kind(), Some(OverlayKind::ScreenshotCapture));
    stack.remove_kind(OverlayKind::ScreenshotCapture);
    assert_eq!(stack.current_kind(), Some(OverlayKind::WindowSizeWarning));
    stack.remove_kind(OverlayKind::WindowSizeWarning);
    assert_eq!(stack.current_kind(), Some(OverlayKind::Screensaver));
  }

  #[test]
  fn game_warning_is_between_screensaver_and_window_size_warning() {
    let mut stack = OverlayStackState::new();
    stack.push(overlay(OverlayKind::Screensaver));
    stack.push(overlay(OverlayKind::GameWarning));
    assert_eq!(stack.current_kind(), Some(OverlayKind::GameWarning));
    stack.push(overlay(OverlayKind::WindowSizeWarning));
    assert_eq!(stack.current_kind(), Some(OverlayKind::WindowSizeWarning));
  }

  #[test]
  fn pushing_same_overlay_kind_replaces_old_state() {
    let mut stack = OverlayStackState::new();
    stack.push(overlay(OverlayKind::LanguageLoading));
    stack.push(overlay(OverlayKind::LanguageLoading));

    assert_eq!(stack.stack.len(), 1);
    assert_eq!(stack.current_kind(), Some(OverlayKind::LanguageLoading));
  }

  #[test]
  fn same_priority_uses_last_pushed_as_current() {
    let mut stack = OverlayStackState::new();
    stack.push(overlay(OverlayKind::LanguageLoading));
    stack.push(overlay(OverlayKind::ClearWarning));

    assert_eq!(stack.current_kind(), Some(OverlayKind::ClearWarning));

    stack.remove_kind(OverlayKind::ClearWarning);
    assert_eq!(stack.current_kind(), Some(OverlayKind::LanguageLoading));
  }

  #[test]
  fn lifecycle_transitions_only_follow_empty_stack_boundaries() {
    let mut stack = OverlayStackState::new();
    stack.push(overlay(OverlayKind::Screensaver));
    stack.push(overlay(OverlayKind::WindowSizeWarning));
    assert_eq!(
      stack.drain_transitions(),
      vec![OverlayStackTransition::Started]
    );

    stack.remove_kind(OverlayKind::WindowSizeWarning);
    assert!(stack.drain_transitions().is_empty());
    stack.remove_kind(OverlayKind::Screensaver);
    assert_eq!(
      stack.drain_transitions(),
      vec![OverlayStackTransition::Stopped]
    );
  }
}
