use crate::host_engine::core::{RuntimeAction, RuntimeSession};
use crossterm::event::KeyCode;

pub fn key_to_runtime_action(key: KeyCode, session: &RuntimeSession) -> Option<RuntimeAction> {
  match key {
    KeyCode::Esc => {
      if session.is_overlay_active() {
        Some(RuntimeAction::CloseOverlay)
      } else {
        Some(RuntimeAction::RequestStop)
      }
    }
    _ => None,
  }
}
