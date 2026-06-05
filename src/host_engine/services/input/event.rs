use super::{KeyboardInputEvent, MouseInputEvent, WindowInputEvent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent {
  Keyboard(KeyboardInputEvent),
  Mouse(MouseInputEvent),
  Window(WindowInputEvent),
}
