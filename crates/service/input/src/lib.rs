//! Keyboard and terminal event collection, frame state, and ordered action dispatch.
//!
//! # Examples
//!
//! ```rust
//! use tg_core_input::{Key, KeyEvent, KeyEventKind};
//! use tg_service_input::InputService;
//! use tg_service_log::LogService;
//!
//! fn main() {
//!   let mut input = InputService::new();
//!   let mut log = LogService::new();
//!
//!   input.queue_key_event(
//!     KeyEvent {
//!       key: Key::A,
//!       kind: KeyEventKind::Press,
//!     },
//!     &mut log,
//!   );
//!   input.poll();
//!
//!   assert!(input.is_down(Key::A));
//!   assert!(input.was_pressed(Key::A));
//!   println!("input ok: key A pressed");
//! }
//! ```

mod service;

pub use service::{InputListenerError, InputNotification, InputService};
pub use tg_core_input::{ActionMapEntry, translate_action_map};
pub use tg_core_input::{
  InputActionEvent, InputEventType, Key, KeyEvent, KeyEventKind, KeyState, RawKeyEvent,
};
pub use tg_core_input::{
  MouseButton, MouseEvent, MouseEventKind, ScrollDirection, SystemEvent, TerminalKeyCode,
  TerminalKeyEvent,
};
pub use tg_core_input::{canonical_key_token, format_key_display, key_token};
