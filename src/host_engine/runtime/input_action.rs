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
    // 临时测试映射，后续删除
    KeyCode::F(1) => Some(RuntimeAction::PushDebugOverlay),
    KeyCode::F(2) => Some(RuntimeAction::PopDebugOverlay),
    _ => None,
  }
}
