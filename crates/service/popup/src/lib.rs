//! Timed host notifications with replaceable text and view state.
//!
//! # Examples
//!
//! ```rust
//! use std::time::Duration;
//!
//! use tg_core_style::{TerminalColor, TextColor};
//! use tg_service_popup::{PopupDismissEvent, PopupRequest, PopupService};
//!
//! fn main() {
//!   let mut popups = PopupService::new();
//!   assert!(popups.show(PopupRequest {
//!     text: "saved".to_string(),
//!     color: TextColor::Terminal(TerminalColor::Green),
//!     duration: Duration::from_secs(2),
//!     dismiss_on: vec![PopupDismissEvent::UiInput],
//!     replaceable: true,
//!     persistent: false,
//!   }));
//!   let view = popups.view().expect("popup is visible");
//!   assert!(
//!     !popups.dismiss(PopupDismissEvent::UiInput),
//!     "too early to dismiss"
//!   );
//!   popups.update(Duration::from_secs(1));
//!   assert!(popups.dismiss(PopupDismissEvent::UiInput));
//!   assert!(popups.view().is_none());
//!   println!("popup ok: {}", view.text);
//! }
//! ```

use std::time::Duration;

use tg_core_style::TextColor;

const CONDITIONAL_DISMISS_DELAY: Duration = Duration::from_millis(500);

/// A popup dismiss event payload queued for its owning consumer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupDismissEvent {
  /// A UI input notification delivered to the owning consumer.
  UiInput,
  /// A screenshot mode input notification delivered to the owning consumer.
  ScreenshotModeInput,
  /// A screenshot operation input notification delivered to the owning consumer.
  ScreenshotOperationInput,
  /// A media rename resolved notification delivered to the owning consumer.
  MediaRenameResolved,
  /// A recording control notification delivered to the owning consumer.
  RecordingControl,
}

/// The popup request representation used by this module.
///
/// # Fields
///
/// * `text` - The text to process or display.
/// * `color` - The color assigned to the target property.
/// * `duration` - The duration represented as a duration.
/// * `dismiss_on` - The ordered dismiss on retained by this owner.
/// * `replaceable` - The replaceable.
/// * `persistent` - The persistent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopupRequest {
  /// The text to process or display.
  pub text: String,
  /// The color assigned to the target property.
  pub color: TextColor,
  /// The duration represented as a duration.
  pub duration: Duration,
  /// The ordered dismiss on retained by this owner.
  pub dismiss_on: Vec<PopupDismissEvent>,

  /// The replaceable.
  pub replaceable: bool,

  /// The persistent.
  pub persistent: bool,
}

/// The popup view representation used by this module.
///
/// # Fields
///
/// * `text` - The text to process or display.
/// * `color` - The color assigned to the target property.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopupView {
  /// The text to process or display.
  pub text: String,
  /// The color assigned to the target property.
  pub color: TextColor,
}

#[derive(Clone, Debug)]
struct ActivePopup {
  request: PopupRequest,
  elapsed: Duration,
}

/// The public entry point for popup operations.
pub struct PopupService {
  active: Option<ActivePopup>,
}

impl PopupService {
  /// Create a popup service with its initial state.
  pub fn new() -> Self {
    Self { active: None }
  }

  /// Display a popup when the current notification permits replacement.
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

  /// Advance popup state using the supplied frame timing.
  pub fn update(&mut self, dt: Duration) {
    let Some(active) = &mut self.active else {
      return;
    };
    if active.request.persistent {
      return;
    }
    active.elapsed = active.elapsed.saturating_add(dt);
    if active.elapsed >= active.request.duration {
      self.active = None;
    }
  }

  /// Dismiss the popup when its configured event and minimum display time permit it.
  pub fn dismiss(&mut self, event: PopupDismissEvent) -> bool {
    if !self.active.as_ref().is_some_and(|active| {
      active.elapsed >= CONDITIONAL_DISMISS_DELAY && active.request.dismiss_on.contains(&event)
    }) {
      return false;
    }
    self.active = None;
    true
  }

  /// Remove the currently displayed popup without waiting for a dismiss event.
  pub fn clear(&mut self) {
    self.active = None;
  }

  /// Return the current view.
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
      persistent: false,
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

  #[test]
  fn persistent_popup_does_not_expire_or_accept_replacement() {
    let mut service = PopupService::new();
    let mut persistent = request("stopping", false);
    persistent.persistent = true;
    persistent.dismiss_on.clear();

    assert!(service.show(persistent));
    service.update(Duration::MAX);

    assert_eq!(service.view().unwrap().text, "stopping");
    assert!(!service.dismiss(PopupDismissEvent::RecordingControl));
    assert!(!service.show(request("replacement", true)));
  }
}
