use crossterm::event::KeyModifiers;
use super::PhysicalKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardInputKind {
  Press,
  Release,
  Repeat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyboardInputEvent {
  pub key: PhysicalKey,
  pub modifiers: KeyModifiers,
  pub kind: KeyboardInputKind,
}

impl KeyboardInputEvent {
  pub fn new(key: PhysicalKey, modifiers: KeyModifiers, kind: KeyboardInputKind) -> Self {
    Self {
      key,
      modifiers,
      kind,
    }
  }
}
