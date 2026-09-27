//! Minimal entry: shows a popup, dismisses it with a matching event after the minimum
//! display delay and checks it is gone.

use std::time::Duration;

use tg_core_style::{TerminalColor, TextColor};
use tg_service_popup::{PopupDismissEvent, PopupRequest, PopupService};

fn main() {
  let mut popups = PopupService::new();
  assert!(popups.show(PopupRequest {
    text: "saved".to_string(),
    color: TextColor::Terminal(TerminalColor::Green),
    duration: Duration::from_secs(2),
    dismiss_on: vec![PopupDismissEvent::UiInput],
    replaceable: true,
    persistent: false,
  }));
  let view = popups.view().expect("popup is visible");
  assert!(!popups.dismiss(PopupDismissEvent::UiInput), "too early to dismiss");
  popups.update(Duration::from_secs(1));
  assert!(popups.dismiss(PopupDismissEvent::UiInput));
  assert!(popups.view().is_none());
  println!("popup ok: {}", view.text);
}
