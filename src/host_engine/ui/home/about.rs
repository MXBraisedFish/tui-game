use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, InputService, KeyState, LayoutService, Rect, RenderService, TerminalColor,
  TextColor, TextStyle, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

pub struct InputDemoUi {
  objects: UiObjectPool,
  areas: [HitAreaId; 2],
  raw_events: Vec<String>,
  raw_event_count: usize,
  action_event_count: usize,
  hit_event_count: usize,
  click_count: usize,
  last_action: String,
  last_hit_event: String,
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
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    let options = HitAreaOptions {
      hover_move: true,
      drag: true,
    };
    let areas = [
      hit_area.create(&mut objects, options),
      hit_area.create(&mut objects, options),
    ];
    Self {
      objects,
      areas,
      raw_events: Vec::new(),
      raw_event_count: 0,
      action_event_count: 0,
      hit_event_count: 0,
      click_count: 0,
      last_action: "None".to_string(),
      last_hit_event: "None".to_string(),
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

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<InputDemoCommand> {
    match event {
      UiEvent::Action(event) if event.state == KeyState::Pressed => {
        let command = match event.action.as_str() {
          "input_demo.capture" => InputDemoCommand::ToggleCapture,
          "input_demo.back" => InputDemoCommand::Back,
          _ => return None,
        };
        self.action_event_count += 1;
        self.last_action = event.action.clone();
        Some(command)
      }
      UiEvent::HitArea(event) => {
        self.hit_event_count += 1;
        self.click_count += usize::from(matches!(event, HitAreaEvent::Click { .. }));
        self.last_hit_event = format_hit_event(event);
        None
      }
      _ => None,
    }
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
    if self.raw_events.len() > 5 {
      self.raw_events.drain(..self.raw_events.len() - 5);
    }
  }

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    input: &InputService,
    hit_area: &HitAreaService,
  ) {
    let width = 72;
    let x = layout.resolve_x(LayoutService::ALIGN_CENTER, width, 0);
    render.draw_text(
      canvas,
      &DrawTextParams::new(x, 2, "Raw rdev + HitArea Demo"),
    );
    render.draw_text(
      canvas,
      &DrawTextParams::new(
        x,
        4,
        format!(
          "Raw capture: {}  raw: {}  actions: {}",
          input.is_raw_key_capture_enabled(),
          self.raw_event_count,
          self.action_event_count
        ),
      ),
    );
    render.draw_text(canvas, &DrawTextParams::new(x, 6, "Recent raw key events:"));
    for (row, event) in self.raw_events.iter().enumerate() {
      render.draw_text(canvas, &DrawTextParams::new(x, 7 + row as u16, event));
    }

    let area_a = Rect {
      x: x + 38,
      y: 6,
      width: 20,
      height: 6,
    };
    let area_b = Rect {
      x: x + 48,
      y: 9,
      width: 20,
      height: 6,
    };
    draw_area(canvas, area_a, "Area A", TerminalColor::Blue);
    hit_area.render(&mut self.objects, self.areas[0], area_a);
    draw_area(canvas, area_b, "Area B (top)", TerminalColor::Red);
    hit_area.render(&mut self.objects, self.areas[1], area_b);

    render.draw_text(
      canvas,
      &DrawTextParams::new(
        x,
        16,
        format!(
          "Hit events: {}  Clicks: {}",
          self.hit_event_count, self.click_count
        ),
      ),
    );
    render.draw_text(
      canvas,
      &DrawTextParams::new(x, 17, format!("Last: {}", self.last_hit_event)),
    );
    for (row, text) in [
      (19, "Enter toggles raw capture; action map remains active"),
      (20, "Test hover, all mouse buttons, click and drag on A/B"),
      (
        21,
        "The overlap belongs to B because B renders last; Esc returns",
      ),
    ] {
      render.draw_text(canvas, &DrawTextParams::new(x, row, text));
    }
  }
}

fn draw_area(canvas: &mut CanvasService, rect: Rect, label: &str, color: TerminalColor) {
  let style = TextStyle {
    foreground: Some(TextColor::Terminal(TerminalColor::BrightWhite)),
    background: Some(TextColor::Terminal(color)),
    ..Default::default()
  };
  for row in 0..rect.height {
    canvas.styled_text(
      rect.x,
      rect.y + row,
      &" ".repeat(rect.width as usize),
      style.clone(),
    );
  }
  canvas.styled_text(rect.x + 1, rect.y + 1, label, style);
}

fn format_hit_event(event: &HitAreaEvent) -> String {
  match event {
    HitAreaEvent::HoverEnter { id, x, y } => format!("Enter {id:?} @({x},{y})"),
    HitAreaEvent::HoverMove { id, x, y } => format!("Move {id:?} @({x},{y})"),
    HitAreaEvent::HoverLeave { id, x, y } => format!("Leave {id:?} @({x},{y})"),
    HitAreaEvent::Press { id, button, x, y } => {
      format!("Press {id:?} {button:?} @({x},{y})")
    }
    HitAreaEvent::Release { id, button, x, y } => {
      format!("Release {id:?} {button:?} @({x},{y})")
    }
    HitAreaEvent::Click { id, button, x, y } => {
      format!("Click {id:?} {button:?} @({x},{y})")
    }
    HitAreaEvent::Drag {
      id,
      button,
      x,
      y,
      dx,
      dy,
    } => {
      format!("Drag {id:?} {button:?} @({x},{y}) d({dx},{dy})")
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{InputActionEvent, InputEventType};

  fn action(name: &str) -> UiEvent {
    UiEvent::Action(InputActionEvent {
      event_type: InputEventType::Keyboard,
      action: name.to_string(),
      state: KeyState::Pressed,
    })
  }

  #[test]
  fn demo_toggles_capture_without_disabling_action_handling() {
    let hit_area = HitAreaService::new();
    let mut input = InputService::new();
    let mut demo = InputDemoUi::init(&hit_area);
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
  }

  #[test]
  fn leaving_demo_disables_capture() {
    let hit_area = HitAreaService::new();
    let mut input = InputService::new();
    let mut demo = InputDemoUi::init(&hit_area);
    input.enable_raw_key_capture();
    demo.leave(&mut input);
    assert!(!input.is_raw_key_capture_enabled());
  }
}
