use crate::host_engine::core::RuntimeAction;
use crate::host_engine::services::{
  KeyboardActionBinding, KeyboardActionMap, KeyboardActionTrigger, KeyboardFrameState,
};

use crossterm::event::KeyCode;

pub fn resolve_runtime_keyboard_actions(state: &KeyboardFrameState) -> Vec<RuntimeAction> {
  let map = runtime_keyboard_action_map();
  map.resolve(state)
}

fn runtime_keyboard_action_map() -> KeyboardActionMap<RuntimeAction> {
  let mut map = KeyboardActionMap::new();

  map.add_binding(KeyboardActionBinding::new(
    KeyCode::Esc,
    KeyboardActionTrigger::Pressed,
    RuntimeAction::Cancel,
  ));

  // 临时测试映射，后续删除
  map.add_binding(KeyboardActionBinding::new(
    KeyCode::F(1),
    KeyboardActionTrigger::Pressed,
    RuntimeAction::PushDebugOverlay,
  ));

  map.add_binding(KeyboardActionBinding::new(
    KeyCode::F(2),
    KeyboardActionTrigger::Pressed,
    RuntimeAction::PopDebugOverlay,
  ));

  map
}
