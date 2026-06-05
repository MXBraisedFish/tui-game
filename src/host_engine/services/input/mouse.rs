#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseInputKind {
  Move,
  Down,
  Up,
  ScrollUp,
  ScrollDown,
  ScrollLeft,
  ScrollRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseInputEvent {
  pub x: u16,
  pub y: u16,
  pub kind: MouseInputKind,
}

impl MouseInputEvent {
  pub fn new(x: u16, y: u16, kind: MouseInputKind) -> Self {
    Self { x, y, kind }
  }
}
