use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardInputKind {
  Press,
  Release,
  Repeat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyboardInputEvent {
  pub code: KeyCode,
  pub modifiers: KeyModifiers,
  pub kind: KeyboardInputKind,
}

impl KeyboardInputEvent {
  pub fn new(code: KeyCode, modifiers: KeyModifiers, kind: KeyboardInputKind) -> Self {
    Self {
      code,
      modifiers,
      kind,
    }
  }
}
