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

  map.add_binding(KeyboardActionBinding::with_priority(
    KeyCode::Esc,
    KeyboardActionTrigger::Pressed,
    RuntimeAction::Cancel,
    100,
  ));

  // 临时测试映射，后续删除
  map.add_binding(KeyboardActionBinding::with_priority(
    KeyCode::F(1),
    KeyboardActionTrigger::Pressed,
    RuntimeAction::PushDebugOverlay,
    10,
  ));

  map.add_binding(KeyboardActionBinding::with_priority(
    KeyCode::F(2),
    KeyboardActionTrigger::Pressed,
    RuntimeAction::PopDebugOverlay,
    10,
  ));

  map
}
