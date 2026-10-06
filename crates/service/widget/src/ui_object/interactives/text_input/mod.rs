//! Grapheme-safe editing, focus, selection, cursor layout, and input rendering.

mod buffer;
mod input;
mod layout;
mod render;
mod service;
mod state;
mod types;

pub use service::TextInputService;
pub(crate) use state::TextInputObjects;
pub use types::{
  TextInputCursorShape, TextInputEvent, TextInputId, TextInputMode, TextInputOptions,
  TextInputRenderParams, VerticalAlign,
};
