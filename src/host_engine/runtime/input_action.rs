use crate::host_engine::core::{RuntimeAction, RuntimeSession};
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

pub fn resolve_runtime_keyboard_actions(state: &KeyboardFrameState, session: &RuntimeSession) -> Vec<RuntimeAction> {
  let resolver = runtime_keyboard_action_resolver(session);
  resolver.resolve(state)
}

fn runtime_keyboard_action_resolver(session: &RuntimeSession) -> KeyboardActionResolver<RuntimeAction> {
  let mut resolver = KeyboardActionResolver::new();

  resolver.add_layer(KeyboardActionLayer::new(
    KeyboardActionLayerKind::Root,
    1000,
    runtime_root_keyboard_action_map(),
  ));

  if session.is_overlay_active() {
    resolver.add_layer(KeyboardActionLayer::new(
      KeyboardActionLayerKind::Overlay,
      900,
      runtime_overlay_keyboard_action_map(),
    ));
  } else {
    resolver.add_layer(KeyboardActionLayer::new(
      KeyboardActionLayerKind::Surface,
      100,
      runtime_base_keyboard_action_map(),
    ));
  }

  resolver
}

fn runtime_root_keyboard_action_map() -> KeyboardActionMap<RuntimeAction> {
  let mut map = KeyboardActionMap::new();

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

fn runtime_overlay_keyboard_action_map() -> KeyboardActionMap<RuntimeAction> {
  let mut map = KeyboardActionMap::new();

  map.add_binding(KeyboardActionBinding::with_options(
    KeyCode::Esc,
    KeyboardActionTrigger::Pressed,
    RuntimeAction::CloseOverlay,
    100,
    true,
  ));

  map
}

fn runtime_base_keyboard_action_map() -> KeyboardActionMap<RuntimeAction> {
  let mut map = KeyboardActionMap::new();

  map.add_binding(KeyboardActionBinding::with_options(
    KeyCode::Esc,
    KeyboardActionTrigger::Pressed,
    RuntimeAction::Cancel,
    100,
    true,
  ));

  map
}
