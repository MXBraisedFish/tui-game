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
  pub required_width: u64,
  pub required_height: u64,
}

impl OverlayStackState {
  pub fn new() -> Self {
    Self {
      stack: Vec::new(),
      transitions: Vec::new(),
    }
  }

  pub fn top(&self) -> Option<&OverlayState> {
    let index = self.current_index()?;
    self.stack.get(index)
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

  /// 主动显示一个覆盖层。同类型覆盖层保持单实例，并成为同层最新项。
  pub fn push(&mut self, overlay: OverlayState) {
    let was_empty = self.stack.is_empty();
    self.stack.retain(|item| item.kind != overlay.kind);
    self.stack.push(overlay);
    if was_empty {
      self.transitions.push(OverlayStackTransition::Started);
    }
  }

  /// 关闭当前显示的覆盖层。
  pub fn pop(&mut self) -> Option<OverlayState> {
    let index = self.current_index()?;
    let overlay = self.stack.remove(index);
    if self.stack.is_empty() {
      self.transitions.push(OverlayStackTransition::Stopped);
    }
    Some(overlay)
  }

  pub fn current_kind(&self) -> Option<OverlayKind> {
    self.top().map(|overlay| overlay.kind)
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

  pub fn get_mut(&mut self, kind: OverlayKind) -> Option<&mut OverlayState> {
    self.stack.iter_mut().find(|overlay| overlay.kind == kind)
  }

  pub fn drain_transitions(&mut self) -> Vec<OverlayStackTransition> {
    std::mem::take(&mut self.transitions)
  }
}

impl OverlayKind {
  pub fn is_program_overlay(self) -> bool {
    matches!(
      self,
      OverlayKind::CoverContinue
        | OverlayKind::ClearWarning
        | OverlayKind::ExportLoading
        | OverlayKind::ExportSettings
        | OverlayKind::GameWarning
        | OverlayKind::LanguageLoading
    )
  }

  fn priority(self) -> u8 {
    match self {
      OverlayKind::CoverContinue => 20,
      OverlayKind::ClearWarning => 20,
      OverlayKind::ExportLoading => 20,
      OverlayKind::ExportSettings => 20,
      OverlayKind::GameWarning => 20,
      OverlayKind::LanguageLoading => 20,
      OverlayKind::Screensaver => 25,
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
  fn fixed_overlay_priority_does_not_depend_on_insertion_order() {
    let mut stack = OverlayStackState::new();
    stack.push(overlay(OverlayKind::ScreenshotCapture));
    stack.push(overlay(OverlayKind::WindowSizeWarning));
    stack.push(overlay(OverlayKind::Screensaver));
    stack.push(overlay(OverlayKind::GameWarning));

    assert_eq!(stack.current_kind(), Some(OverlayKind::ScreenshotCapture));
    assert_eq!(stack.top().map(|item| item.kind), stack.current_kind());

    assert_eq!(
      stack.pop().map(|item| item.kind),
      Some(OverlayKind::ScreenshotCapture)
    );
    assert_eq!(stack.current_kind(), Some(OverlayKind::WindowSizeWarning));
    assert_eq!(
      stack.pop().map(|item| item.kind),
      Some(OverlayKind::WindowSizeWarning)
    );
    assert_eq!(stack.current_kind(), Some(OverlayKind::Screensaver));
    assert_eq!(
      stack.pop().map(|item| item.kind),
      Some(OverlayKind::Screensaver)
    );
    assert_eq!(stack.current_kind(), Some(OverlayKind::GameWarning));
  }

  #[test]
  fn game_warning_is_an_ordinary_overlay_below_screensaver() {
    let mut stack = OverlayStackState::new();
    stack.push(overlay(OverlayKind::Screensaver));
    stack.push(overlay(OverlayKind::GameWarning));

    assert_eq!(stack.current_kind(), Some(OverlayKind::Screensaver));
  }

  #[test]
  fn every_ordinary_overlay_shares_the_same_priority() {
    let ordinary = [
      OverlayKind::CoverContinue,
      OverlayKind::ClearWarning,
      OverlayKind::ExportLoading,
      OverlayKind::ExportSettings,
      OverlayKind::GameWarning,
      OverlayKind::LanguageLoading,
    ];

    for earlier in ordinary {
      assert!(earlier.is_program_overlay());
      for later in ordinary {
        if earlier == later {
          continue;
        }
        let mut stack = OverlayStackState::new();
        stack.push(overlay(earlier));
        stack.push(overlay(later));
        assert_eq!(stack.current_kind(), Some(later));
      }
    }
    assert!(!OverlayKind::Screensaver.is_program_overlay());
    assert!(!OverlayKind::WindowSizeWarning.is_program_overlay());
    assert!(!OverlayKind::ScreenshotCapture.is_program_overlay());
  }

  #[test]
  fn refreshing_overlay_state_does_not_change_active_order() {
    let mut stack = OverlayStackState::new();
    stack.push(overlay(OverlayKind::LanguageLoading));
    stack.push(overlay(OverlayKind::ClearWarning));

    stack
      .get_mut(OverlayKind::LanguageLoading)
      .unwrap()
      .render
      .required_width = 123;

    assert_eq!(stack.current_kind(), Some(OverlayKind::ClearWarning));
    assert_eq!(stack.top().map(|item| item.kind), stack.current_kind());
    assert_eq!(
      stack
        .get(OverlayKind::LanguageLoading)
        .unwrap()
        .render
        .required_width,
      123
    );
  }

  #[test]
  fn actively_showing_same_overlay_moves_it_to_latest_position_in_its_layer() {
    let mut stack = OverlayStackState::new();
    stack.push(overlay(OverlayKind::LanguageLoading));
    stack.push(overlay(OverlayKind::ClearWarning));
    stack.push(overlay(OverlayKind::LanguageLoading));

    assert_eq!(stack.stack.len(), 2);
    assert_eq!(stack.current_kind(), Some(OverlayKind::LanguageLoading));
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
