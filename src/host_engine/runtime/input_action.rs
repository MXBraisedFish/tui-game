use crate::host_engine::core::{ExecutionContext, HostSurface, RuntimeAction, UiNode};
use crate::host_engine::services::{
  KeyboardActionBinding,
  KeyboardActionLayer,
  KeyboardActionLayerKind,
  KeyboardActionMap,
  KeyboardActionResolver,
  KeyboardActionTrigger,
  KeyboardFrameState,
};

use super::input_context::RuntimeKeyboardContext;
use crossterm::event::KeyCode;

pub fn resolve_runtime_keyboard_actions(state: &KeyboardFrameState, context: RuntimeKeyboardContext) -> Vec<RuntimeAction> {
  let resolver = runtime_keyboard_action_resolver(context);
  resolver.resolve(state)
}

fn runtime_keyboard_action_resolver(context: RuntimeKeyboardContext) -> KeyboardActionResolver<RuntimeAction> {
  let mut resolver = KeyboardActionResolver::new();

  resolver.add_layer(KeyboardActionLayer::new(
    KeyboardActionLayerKind::Root,
    1000,
    runtime_root_keyboard_action_map(),
  ));

  if context.is_overlay_active() {
    resolver.add_layer(KeyboardActionLayer::new(
      KeyboardActionLayerKind::Overlay,
      900,
      runtime_overlay_keyboard_action_map(),
    ));
  } else {
    resolver.add_layer(KeyboardActionLayer::new(
      KeyboardActionLayerKind::UiNode,
      300,
      runtime_ui_node_keyboard_action_map(context.current_ui_node()),
    ));

    resolver.add_layer(KeyboardActionLayer::new(
      KeyboardActionLayerKind::Surface,
      200,
      runtime_surface_keyboard_action_map(context.host_surface()),
    ));

    resolver.add_layer(KeyboardActionLayer::new(
      KeyboardActionLayerKind::ExecutionContext,
      100,
      runtime_execution_context_keyboard_action_map(context.execution_context()),
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

fn runtime_ui_node_keyboard_action_map(node: Option<UiNode>) -> KeyboardActionMap<RuntimeAction> {
  match node {
    Some(UiNode::Root) | None => KeyboardActionMap::new(),
  }
}

fn runtime_surface_keyboard_action_map(surface: HostSurface) -> KeyboardActionMap<RuntimeAction> {
  match surface {
    HostSurface::MainMenu => main_menu_keyboard_action_map(),
  }
}

fn runtime_execution_context_keyboard_action_map(context: ExecutionContext) -> KeyboardActionMap<RuntimeAction> {
  match context {
    ExecutionContext::Host => KeyboardActionMap::new(),
  }
}

fn main_menu_keyboard_action_map() -> KeyboardActionMap<RuntimeAction> {
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
