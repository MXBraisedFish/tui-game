mod event;
mod keyboard;
mod mouse;
mod service;
mod window;

pub use event::InputEvent;
pub use keyboard::{KeyboardInputEvent, KeyboardInputKind};
pub use mouse::{MouseInputEvent, MouseInputKind};
pub use service::{InputService, KeyEventKind, KeyInput};
pub use window::WindowInputEvent;
