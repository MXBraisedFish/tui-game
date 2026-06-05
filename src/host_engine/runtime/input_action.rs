use crate::host_engine::core::RuntimeAction;
use crate::host_engine::services::{
  KeyboardActionBinding,
  KeyboardActionLayer,
  KeyboardActionLayerKind,
  KeyboardActionMap,
  KeyboardActionResolver,
  KeyboardActionTrigger,
  KeyboardFrameState,
};

use crossterm::event::KeyCode;

pub fn resolve_runtime_keyboard_actions(state: &KeyboardFrameState) -> Vec<RuntimeAction> {
  let resolver = runtime_keyboard_action_resolver();
  resolver.resolve(state)
}

fn runtime_keyboard_action_resolver() -> KeyboardActionResolver<RuntimeAction> {
  let mut resolver = KeyboardActionResolver::new();

  resolver.add_layer(KeyboardActionLayer::new(
    KeyboardActionLayerKind::Root,
    1000,
    runtime_keyboard_action_map(),
  ));

  resolver
}

fn runtime_keyboard_action_map() -> KeyboardActionMap<RuntimeAction> {
  let mut map = KeyboardActionMap::new();

  map.add_binding(KeyboardActionBinding::with_options(
    KeyCode::Esc,
    KeyboardActionTrigger::Pressed,
    RuntimeAction::Cancel,
    100,
    true,
  ));

  // 临时测试映射，后续删除
  map.add_binding(KeyboardActionBinding::with_options(
    KeyCode::F(1),
    KeyboardActionTrigger::Pressed,
    RuntimeAction::PushDebugOverlay,
    10,
    true,
  ));

  map.add_binding(KeyboardActionBinding::with_options(
    KeyCode::F(2),
    KeyboardActionTrigger::Pressed,
    RuntimeAction::PopDebugOverlay,
    10,
    true,
  ));

  map
}
