#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowInputEvent {
  Resize { width: u16, height: u16 },
  FocusGained,
  FocusLost,
}
