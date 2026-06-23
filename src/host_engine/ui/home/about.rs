use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, InputActionEvent, KeyState, LayoutService, Rect,
  RenderService, TerminalColor, TextColor, TextInputCursorShape, TextInputEvent, TextInputId,
  TextInputMode, TextInputOptions, TextInputRenderParams, TextInputService, TextStyle,
  UiObjectPool, UiObjectPoolOwner, VerticalAlign,
};

pub struct InputDemoUi {
  objects: UiObjectPool,
  inputs: [TextInputId; 4],
  selected: usize,
  last_event: String,
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
  SelectPrevious,
  SelectNext,
  FocusInput,
  Back,
}

impl InputDemoUi {
  pub fn init(text_input: &TextInputService) -> Self {
    let mut objects = UiObjectPool::new();
    let modes = [
      TextInputMode::SingleLine,
      TextInputMode::MultiLine,
      TextInputMode::SingleLine,
      TextInputMode::MultiLine,
    ];
    let samples = ["123 abc", "", "我爱你", "f%<fg:red>raw</fg>\nEmoji: 👨‍👩‍👧‍👦"];
    let inputs = std::array::from_fn(|index| {
      text_input.create(
        &mut objects,
        TextInputOptions {
          initial_text: samples[index].to_string(),
          max_chars: Some(128),
          mode: modes[index],
        },
      )
    });
    Self {
      objects,
      inputs,
      selected: 0,
      last_event: "None".to_string(),
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "input_demo.previous".to_string(),
        description: "Select previous text input".to_string(),
        keys: vec![vec!["up".to_string()]],
      },
      ActionMapEntry {
        action: "input_demo.next".to_string(),
        description: "Select next text input".to_string(),
        keys: vec![vec!["down".to_string()]],
      },
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
      "input_demo.previous" => Some(InputDemoCommand::SelectPrevious),
      "input_demo.next" => Some(InputDemoCommand::SelectNext),
      "input_demo.focus" => Some(InputDemoCommand::FocusInput),
      "input_demo.back" => Some(InputDemoCommand::Back),
      _ => None,
    }
  }

  pub fn select_previous(&mut self) {
    self.selected = self
      .selected
      .checked_sub(1)
      .unwrap_or(self.inputs.len() - 1);
  }

  pub fn select_next(&mut self) {
    self.selected = (self.selected + 1) % self.inputs.len();
  }

  pub fn focus(&mut self, text_input: &mut TextInputService) {
    text_input.focus(&mut self.objects, self.inputs[self.selected]);
  }

  pub fn update(&mut self, text_input: &mut TextInputService) {
    for (index, id) in self.inputs.into_iter().enumerate() {
      for event in text_input.take_events(&mut self.objects, id) {
        let name = match &event {
          TextInputEvent::Focused { .. } => "Focused",
          TextInputEvent::Blurred { .. } => "Blurred",
          TextInputEvent::Changed { .. } => "Changed",
          TextInputEvent::Submit { .. } => "Submit",
          TextInputEvent::Cancel { .. } => "Cancel",
        };
        self.last_event = format!("Input {}: {name}", index + 1);

        if matches!(event, TextInputEvent::Cancel { .. }) {
          text_input.blur(&mut self.objects);
        }
      }
    }
  }

  pub fn render(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    text_input: &TextInputService,
  ) -> Option<(u16, u16)> {
    let width = 52;
    let x = layout.resolve_x(LayoutService::ALIGN_CENTER, width, 0);
    let white = Some(TextColor::Terminal(TerminalColor::BrightWhite));
    let gray = Some(TextColor::Terminal(TerminalColor::BrightBlack));
    let background = Some(TextColor::Terminal(TerminalColor::Blue));

    render.draw_text(canvas, &DrawTextParams::new(x, 2, "TextInput Demo"));
    let mut cursor = None;
    for (index, id) in self.inputs.into_iter().enumerate() {
      let (label, y, height, vertical_align, cursor_shape, cursor_blink) = [
        ("Single 1 Block", 4, 1, VerticalAlign::Top, None, true),
        (
          "Multi 1 Underline",
          6,
          1,
          VerticalAlign::Top,
          Some(TextInputCursorShape::Underline),
          true,
        ),
        (
          "Single 3 Line Static",
          8,
          3,
          VerticalAlign::Center,
          Some(TextInputCursorShape::Line),
          false,
        ),
        (
          "Multi 5 None",
          12,
          5,
          VerticalAlign::Top,
          Some(TextInputCursorShape::None),
          false,
        ),
      ][index];
      let marker = if index == self.selected { ">" } else { " " };
      render.draw_text(
        canvas,
        &DrawTextParams::new(x, y, format!("{marker} {label}")),
      );
      cursor = text_input
        .render(
          &self.objects,
          id,
          &TextInputRenderParams {
            rect: Rect {
              x: x + 24,
              y,
              width: 15,
              height,
            },
            placeholder: format!("Type in input {}...", index + 1),
            fg: white.clone(),
            bg: background.clone(),
            placeholder_fg: gray.clone(),
            text_style: TextStyle::default(),
            placeholder_style: TextStyle::default(),
            cursor_style: TextStyle {
              foreground: white.clone(),
              ..Default::default()
            },
            cursor_shape,
            cursor_blink,
            vertical_align,
          },
          canvas,
        )
        .or(cursor);
    }

    let focused = text_input.is_focused(&self.objects, self.inputs[self.selected]);
    for (row, text) in [
      (18, format!("Last Event: {}", self.last_event)),
      (
        19,
        format!("Selected: Input {} / Focused: {focused}", self.selected + 1),
      ),
      (21, "Up/Down: Select  Enter: Focus/Submit".to_string()),
      (
        22,
        "Ctrl+Enter: Newline (Multi)  Esc: Cancel/Back".to_string(),
      ),
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
    let mut service = TextInputService::new();
    let mut demo = InputDemoUi::init(&service);
    let first = demo.inputs[0];
    assert!(service.clear(demo.objects_mut(), first));
    service.take_events(demo.objects_mut(), first);
    demo.focus(&mut service);
    demo.update(&mut service);

    service.route_terminal_key(
      demo.objects_mut(),
      TerminalKeyEvent {
        code: TerminalKeyCode::Char('a'),
        ctrl: false,
      },
    );
    demo.update(&mut service);
    assert_eq!(service.get_text(&demo.objects, first), Some("a"));
    assert_eq!(demo.last_event, "Input 1: Changed");

    service.route_terminal_key(
      demo.objects_mut(),
      TerminalKeyEvent {
        code: TerminalKeyCode::Enter,
        ctrl: false,
      },
    );
    demo.update(&mut service);
    assert!(service.is_active());
    assert_eq!(demo.last_event, "Input 1: Submit");

    service.route_terminal_key(
      demo.objects_mut(),
      TerminalKeyEvent {
        code: TerminalKeyCode::Esc,
        ctrl: false,
      },
    );
    demo.update(&mut service);
    assert!(!service.is_active());
    assert_eq!(demo.last_event, "Input 1: Cancel");
  }

  #[test]
  fn demo_inputs_keep_independent_text_and_focus() {
    let mut service = TextInputService::new();
    let mut demo = InputDemoUi::init(&service);
    let [first, second, third, fourth] = demo.inputs;

    assert_eq!(
      (first, second, third, fourth),
      (
        TextInputId(1),
        TextInputId(2),
        TextInputId(3),
        TextInputId(4)
      )
    );
    demo.focus(&mut service);
    assert!(service.is_focused(&demo.objects, first));
    assert!(!service.focus(&mut demo.objects, second));
    assert!(service.set_text(&mut demo.objects, first, "first"));
    assert!(service.blur(&mut demo.objects));

    demo.select_next();
    demo.focus(&mut service);
    assert!(service.is_focused(&demo.objects, second));
    assert!(service.set_text(&mut demo.objects, second, "second"));
    assert_eq!(service.get_text(&demo.objects, first), Some("first"));
    assert_eq!(service.get_text(&demo.objects, second), Some("second"));
    assert_eq!(service.get_text(&demo.objects, third), Some("我爱你"));
    assert_eq!(
      service.get_text(&demo.objects, fourth),
      Some("f%<fg:red>raw</fg>\nEmoji: 👨‍👩‍👧‍👦")
    );
  }

  #[test]
  fn demo_multiline_input_accepts_ctrl_enter() {
    let mut service = TextInputService::new();
    let mut demo = InputDemoUi::init(&service);
    demo.select_next();
    demo.focus(&mut service);
    let multiline = demo.inputs[1];

    service.route_terminal_key(
      demo.objects_mut(),
      TerminalKeyEvent {
        code: TerminalKeyCode::Enter,
        ctrl: true,
      },
    );
    demo.update(&mut service);
    assert_eq!(service.get_text(&demo.objects, multiline), Some("\n"));
    assert!(service.is_focused(&demo.objects, multiline));
    assert_eq!(demo.last_event, "Input 2: Changed");
  }
}
