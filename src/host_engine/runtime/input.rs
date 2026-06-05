use crate::host_engine::core::RuntimeWorld;
use crate::host_engine::services::{EngineServices, InputEvent, LogSource, WindowInputEvent};

use super::input_action::resolve_runtime_keyboard_actions;
use super::input_context::RuntimeKeyboardContext;

pub fn handle_runtime_input_event(
  event: InputEvent,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match event {
    InputEvent::Keyboard(_) => {}
    InputEvent::Window(window) => {
      handle_runtime_window_event(window, services, world);
    }
    InputEvent::Mouse(_) => {}
  }
}

pub fn handle_runtime_keyboard_actions(services: &EngineServices, world: &mut RuntimeWorld) {
  if !world.session.should_accept_keyboard_input() {
    return;
  }

  let context = RuntimeKeyboardContext::from_session(&world.session);
  let actions = resolve_runtime_keyboard_actions(services.input.keyboard_state(), context);

  for action in actions {
    world.session.handle_runtime_action(action);
  }
}

fn handle_runtime_window_event(event: WindowInputEvent, services: &mut EngineServices, world: &mut RuntimeWorld) {
  match event {
    WindowInputEvent::Resize { width, height } => {
      services.canvas.resize(width, height);
      services.ui.on_resize(width, height);
      services.log.info(
        LogSource::Runtime,
        format!("[Terminal Resize detected: {}x{}]", width, height),
      );
    }
    WindowInputEvent::FocusGained => {
      world.session.set_terminal_focused(true);
    }
    WindowInputEvent::FocusLost => {
      world.session.set_terminal_focused(false);
    }
  }
}
