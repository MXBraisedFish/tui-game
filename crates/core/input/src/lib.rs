//! Input data model: keys, key patterns/bindings, action maps, key tokens and terminal system events.

mod action_map;
mod events;
mod key;
mod key_token;

pub use action_map::{ActionMapEntry, ActionMapTranslateError, translate_action_map};
pub use events::{
  FocusEvent, MouseButton, MouseEvent, MouseEventKind, ResizeEvent, ScrollDirection, SystemEvent,
  TerminalKeyCode, TerminalKeyEvent,
};
pub use key::{
  InputActionEvent, InputEventType, Key, KeyBinding, KeyEvent, KeyEventKind, KeyPattern, KeyState,
  RawKeyEvent,
};
pub use key_token::{canonical_key_token, display_key_token, format_key_display, key_token, parse_key_token};
