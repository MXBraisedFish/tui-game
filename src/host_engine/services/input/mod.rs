mod service;

pub use tg_core_input::{ActionMapEntry, translate_action_map};
pub use tg_core_input::{
  MouseButton, MouseEvent, MouseEventKind, ScrollDirection, SystemEvent, TerminalKeyCode,
  TerminalKeyEvent,
};
pub use tg_core_input::{canonical_key_token, format_key_display, key_token};
pub use tg_core_input::{
  InputActionEvent, InputEventType, Key, KeyEvent, KeyEventKind, KeyState, RawKeyEvent,
};
pub use service::InputService;
