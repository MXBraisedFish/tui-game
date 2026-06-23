use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, InputActionEvent, InputService, KeyState,
  LayoutService, RenderService, UiObjectPool, UiObjectPoolOwner,
};

pub struct InputDemoUi {
  objects: UiObjectPool,
  raw_events: Vec<String>,
  raw_event_count: usize,
  action_event_count: usize,
  last_action: String,
}

impl UiObjectPoolOwner for InputDemoUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputDemoCommand {
  ToggleCapture,
  Back,
}

impl InputDemoUi {
  pub fn init() -> Self {
    Self {
      objects: UiObjectPool::new(),
      raw_events: Vec::new(),
      raw_event_count: 0,
      action_event_count: 0,
      last_action: "None".to_string(),
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "input_demo.capture".to_string(),
        description: "Toggle raw rdev key capture".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "input_demo.back".to_string(),
        description: "Back to home".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
    ]
  }

  pub fn handle_event(&mut self, event: &InputActionEvent) -> Option<InputDemoCommand> {
    if event.state != KeyState::Pressed {
      return None;
    }
    let command = match event.action.as_str() {
      "input_demo.capture" => InputDemoCommand::ToggleCapture,
      "input_demo.back" => InputDemoCommand::Back,
      _ => return None,
    };
    self.action_event_count += 1;
    self.last_action = event.action.clone();
    Some(command)
  }

  pub fn toggle_capture(&mut self, input: &mut InputService) {
    if input.is_raw_key_capture_enabled() {
      input.disable_raw_key_capture();
    } else {
      input.enable_raw_key_capture();
      self.raw_events.clear();
      self.raw_event_count = 0;
    }
  }

  pub fn leave(&mut self, input: &mut InputService) {
    if input.is_raw_key_capture_enabled() {
      input.disable_raw_key_capture();
    }
  }

  pub fn update(&mut self, input: &mut InputService) {
    for event in input.take_raw_key_events() {
      self.raw_event_count += 1;
      self.raw_events.push(format!(
        "{}  {:?}  {:?}",
        event.display, event.key, event.kind
      ));
    }
    if self.raw_events.len() > 8 {
      self.raw_events.drain(..self.raw_events.len() - 8);
    }
  }

  pub fn render(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    input: &InputService,
  ) {
    let width = 64;
    let x = layout.resolve_x(LayoutService::ALIGN_CENTER, width, 0);
    render.draw_text(
      canvas,
      &DrawTextParams::new(x, 2, "Raw rdev Key Capture Demo"),
    );
    render.draw_text(
      canvas,
      &DrawTextParams::new(
        x,
        4,
        format!("Capture enabled: {}", input.is_raw_key_capture_enabled()),
      ),
    );
    render.draw_text(
      canvas,
      &DrawTextParams::new(x, 5, format!("Raw events: {}", self.raw_event_count)),
    );
    render.draw_text(
      canvas,
      &DrawTextParams::new(
        x,
        6,
        format!(
          "Action events: {}  Last: {}",
          self.action_event_count, self.last_action
        ),
      ),
    );
    render.draw_text(
      canvas,
      &DrawTextParams::new(x, 8, "Display    Internal Key    State"),
    );
    for (row, event) in self.raw_events.iter().enumerate() {
      render.draw_text(canvas, &DrawTextParams::new(x, 9 + row as u16, event));
    }
    for (row, text) in [
      (18, "Enter: toggle raw capture (action map remains active)"),
      (19, "Esc: return Home and disable raw capture"),
      (20, "Press letters, modifiers, arrows and function keys"),
      (21, "Every raw event exposes Key + display + Press/Release"),
    ] {
      render.draw_text(canvas, &DrawTextParams::new(x, row, text));
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::InputActionEvent;

  fn action(name: &str) -> InputActionEvent {
    InputActionEvent {
      event_type: crate::host_engine::services::InputEventType::Keyboard,
      action: name.to_string(),
      state: KeyState::Pressed,
    }
  }

  #[test]
  fn demo_toggles_capture_without_disabling_action_handling() {
    let mut input = InputService::new();
    let mut demo = InputDemoUi::init();

    assert_eq!(
      demo.handle_event(&action("input_demo.capture")),
      Some(InputDemoCommand::ToggleCapture)
    );
    demo.toggle_capture(&mut input);
    assert!(input.is_raw_key_capture_enabled());
    assert_eq!(
      demo.handle_event(&action("input_demo.capture")),
      Some(InputDemoCommand::ToggleCapture)
    );
    assert_eq!(demo.action_event_count, 2);

    demo.toggle_capture(&mut input);
    assert!(!input.is_raw_key_capture_enabled());
  }

  #[test]
  fn leaving_demo_disables_capture() {
    let mut input = InputService::new();
    let mut demo = InputDemoUi::init();
    input.enable_raw_key_capture();
    demo.leave(&mut input);
    assert!(!input.is_raw_key_capture_enabled());
  }
}
