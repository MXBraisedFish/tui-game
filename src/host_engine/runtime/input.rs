use crate::host_engine::core::RuntimeWorld;
use crate::host_engine::services::{
  EngineServices,
  InputEvent,
  KeyboardInputKind,
  LogSource,
  WindowInputEvent,
};

use super::input_action::key_to_runtime_action;

pub fn handle_runtime_input_event(
  event: InputEvent,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match event {
    InputEvent::Keyboard(key) => {
      handle_runtime_keyboard_event(key.code, key.kind, world);
    }
    InputEvent::Window(window) => {
      handle_runtime_window_event(window, services);
    }
    InputEvent::Mouse(_) => {}
  }
}

fn handle_runtime_keyboard_event(
  code: crossterm::event::KeyCode,
  kind: KeyboardInputKind,
  world: &mut RuntimeWorld,
) {
  if !matches!(kind, KeyboardInputKind::Press | KeyboardInputKind::Repeat) {
    return;
  }

  if let Some(action) = key_to_runtime_action(code, &world.session) {
    world.session.handle_runtime_action(action);
  }
}

fn handle_runtime_window_event(event: WindowInputEvent, services: &mut EngineServices) {
  match event {
    WindowInputEvent::Resize { width, height } => {
      services.canvas.resize(width, height);
      services.ui.on_resize(width, height);
      services.log.info(
        LogSource::Runtime,
        format!("[Terminal Resize detected: {}x{}]", width, height),
      );
    }
    WindowInputEvent::FocusGained => {}
    WindowInputEvent::FocusLost => {}
  }
}
