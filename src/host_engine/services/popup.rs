use std::time::Duration;

use crate::host_engine::services::TextColor;

const CONDITIONAL_DISMISS_DELAY: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupDismissEvent {
  ScreenshotModeInput,
  ScreenshotOperationInput,
  MediaRenameResolved,
  RecordingControl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopupRequest {
  pub text: String,
  pub color: TextColor,
  pub duration: Duration,
  pub dismiss_on: Vec<PopupDismissEvent>,
  /// 当前弹窗是否允许被之后到来的弹窗替换。
  pub replaceable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopupView {
  pub text: String,
  pub color: TextColor,
}

#[derive(Clone, Debug)]
struct ActivePopup {
  request: PopupRequest,
  elapsed: Duration,
}

/// 统一管理短时状态弹窗的覆盖、超时和条件回收。
pub struct PopupService {
  active: Option<ActivePopup>,
}

impl PopupService {
  pub fn new() -> Self {
    Self { active: None }
  }

  pub fn show(&mut self, request: PopupRequest) -> bool {
    if self
      .active
      .as_ref()
      .is_some_and(|active| !active.request.replaceable)
    {
      return false;
    }
    self.active = Some(ActivePopup {
      request,
      elapsed: Duration::ZERO,
    });
    true
  }

  pub fn update(&mut self, dt: Duration) {
    let Some(active) = &mut self.active else {
      return;
    };
    active.elapsed = active.elapsed.saturating_add(dt);
    if active.elapsed >= active.request.duration {
      self.active = None;
    }
  }

  pub fn dismiss(&mut self, event: PopupDismissEvent) -> bool {
    if !self.active.as_ref().is_some_and(|active| {
      active.elapsed >= CONDITIONAL_DISMISS_DELAY && active.request.dismiss_on.contains(&event)
    }) {
      return false;
    }
    self.active = None;
    true
  }

  pub fn clear(&mut self) {
    self.active = None;
  }

  pub fn view(&self) -> Option<PopupView> {
    let active = self.active.as_ref()?;
    Some(PopupView {
      text: active.request.text.clone(),
      color: active.request.color.clone(),
    })
  }
}

impl Default for PopupService {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn request(text: &str, replaceable: bool) -> PopupRequest {
    PopupRequest {
      text: text.to_string(),
      color: TextColor::Rgb { r: 1, g: 2, b: 3 },
      duration: Duration::from_secs(2),
      dismiss_on: vec![PopupDismissEvent::RecordingControl],
      replaceable,
    }
  }

  #[test]
  fn non_replaceable_popup_rejects_new_popup_until_dismissed() {
    let mut service = PopupService::new();
    assert!(service.show(request("first", false)));
    assert!(!service.show(request("second", true)));
    assert_eq!(service.view().unwrap().text, "first");
    service.update(CONDITIONAL_DISMISS_DELAY);
    assert!(service.dismiss(PopupDismissEvent::RecordingControl));
    assert!(service.show(request("second", true)));
  }

  #[test]
  fn popup_expires_after_requested_duration() {
    let mut service = PopupService::new();
    service.show(request("short", true));
    service.update(Duration::from_secs(2));
    assert!(service.view().is_none());
  }
}
