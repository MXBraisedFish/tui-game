use super::{InputEvent, KeyboardInputEvent, MouseInputEvent, WindowInputEvent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawInputSource {
  Terminal,
  GlobalKeyboard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawInputEvent {
  Keyboard {
    source: RawInputSource,
    event: KeyboardInputEvent,
  },
  Mouse {
    source: RawInputSource,
    event: MouseInputEvent,
  },
  Window {
    source: RawInputSource,
    event: WindowInputEvent,
  },
}

impl RawInputEvent {
  pub fn source(&self) -> RawInputSource {
    match self {
      RawInputEvent::Keyboard { source, .. } => *source,
      RawInputEvent::Mouse { source, .. } => *source,
      RawInputEvent::Window { source, .. } => *source,
    }
  }

  pub fn into_input_event(self) -> InputEvent {
    match self {
      RawInputEvent::Keyboard { event, .. } => InputEvent::Keyboard(event),
      RawInputEvent::Mouse { event, .. } => InputEvent::Mouse(event),
      RawInputEvent::Window { event, .. } => InputEvent::Window(event),
    }
  }
}
