//! Terminal text styling primitives: colors, style flags, styled text segments,
//! canvas cells and composed terminal frames.

mod cell;
mod color;
mod frame;
mod style;
mod types;

pub use cell::CanvasCell;
pub use color::parse_text_color;
pub use frame::{ComposedCell, ComposedFrame};
pub use style::{TerminalColor, TextColor, TextStyle};
pub use types::{RichText, RichTextSegment};
