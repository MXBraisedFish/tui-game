use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, InputActionEvent, InputDrawParams, InputEvent,
  InputId, InputOptions, KeyState, LayoutService, Rect, RenderService, TerminalColor,
  TerminalKeyEvent, TextColor, TextInputService, TextStyle, UiObjectPool,
};

pub struct InputDemoUi {
  objects: UiObjectPool,
  input: InputId,
  last_event: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputDemoCommand {
  FocusInput,
  Back,
}

impl InputDemoUi {
  pub fn init() -> Self {
    let mut objects = UiObjectPool::new();
    let input = objects.create_input(InputOptions {
      initial_text: String::new(),
      max_chars: Some(32),
      multiline: false,
    });
    Self {
      objects,
      input,
      last_event: "None".to_string(),
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "input_demo.focus".to_string(),
        description: "Focus text input".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "input_demo.back".to_string(),
        description: "Back to home".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
    ]
  }

  pub fn handle_event(&self, event: &InputActionEvent) -> Option<InputDemoCommand> {
    if event.state != KeyState::Pressed {
      return None;
    }
    match event.action.as_str() {
      "input_demo.focus" => Some(InputDemoCommand::FocusInput),
      "input_demo.back" => Some(InputDemoCommand::Back),
      _ => None,
    }
  }

  pub fn focus(&mut self, text_input: &mut TextInputService) {
    text_input.focus_input(&mut self.objects, self.input);
  }

  pub fn route_terminal_key(&mut self, text_input: &mut TextInputService, key: TerminalKeyEvent) {
    text_input.route_terminal_key(&mut self.objects, key);
  }

  pub fn update(&mut self, text_input: &mut TextInputService) {
    for event in self.objects.take_input_events(self.input) {
      self.last_event = match &event {
        InputEvent::Focused { .. } => "Focused",
        InputEvent::Blurred { .. } => "Blurred",
        InputEvent::Changed { .. } => "Changed",
        InputEvent::Submit { .. } => "Submit",
        InputEvent::Cancel { .. } => "Cancel",
      }
      .to_string();

      if matches!(event, InputEvent::Cancel { .. }) {
        text_input.blur_input(&mut self.objects);
      }
    }
  }

  pub fn render(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    active_input: Option<InputId>,
  ) -> Option<(u16, u16)> {
    let width = 40;
    let x = layout.resolve_x(LayoutService::ALIGN_CENTER, width, 0);
    let cyan = Some(TextColor::Terminal(TerminalColor::BrightCyan));
    let gray = Some(TextColor::Terminal(TerminalColor::BrightBlack));

    render.draw_text(canvas, &DrawTextParams::new(x, 4, "TextInput Demo"));
    let cursor = self.objects.draw_input(
      self.input,
      &InputDrawParams {
        rect: Rect {
          x,
          y: 7,
          width,
          height: 1,
        },
        placeholder: "Type something...".to_string(),
        text_style: TextStyle::default(),
        placeholder_style: TextStyle {
          foreground: gray.clone(),
          ..Default::default()
        },
        cursor_style: TextStyle {
          foreground: cyan,
          ..Default::default()
        },
      },
      canvas,
      active_input,
    );

    let current = self.objects.get_input_text(self.input).unwrap_or("");
    for (row, text) in [
      (10, format!("Current: {current}")),
      (12, format!("Last Event: {}", self.last_event)),
      (14, format!("Active: {}", active_input == Some(self.input))),
      (17, "Controls:".to_string()),
      (18, "  Enter: Focus / Submit".to_string()),
      (19, "  Esc: Cancel / Back".to_string()),
    ] {
      render.draw_text(canvas, &DrawTextParams::new(x, row, text));
    }
    cursor
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{TerminalKeyCode, TerminalKeyEvent};

  #[test]
  fn demo_submit_stays_active_and_cancel_blurs() {
    let mut demo = InputDemoUi::init();
    let mut service = TextInputService::new();
    demo.focus(&mut service);
    demo.update(&mut service);

    demo.route_terminal_key(
      &mut service,
      TerminalKeyEvent {
        code: TerminalKeyCode::Char('a'),
      },
    );
    demo.update(&mut service);
    assert_eq!(demo.objects.get_input_text(demo.input), Some("a"));
    assert_eq!(demo.last_event, "Changed");

    demo.route_terminal_key(
      &mut service,
      TerminalKeyEvent {
        code: TerminalKeyCode::Enter,
      },
    );
    demo.update(&mut service);
    assert!(service.is_active());
    assert_eq!(demo.last_event, "Submit");

    demo.route_terminal_key(
      &mut service,
      TerminalKeyEvent {
        code: TerminalKeyCode::Esc,
      },
    );
    demo.update(&mut service);
    assert!(!service.is_active());
    assert_eq!(demo.last_event, "Cancel");
  }
}
