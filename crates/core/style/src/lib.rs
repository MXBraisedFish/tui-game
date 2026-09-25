//! Terminal text styling primitives: colors, style flags and styled text segments.

mod color;
mod style;
mod types;

pub use color::parse_text_color;
pub use style::{TerminalColor, TextColor, TextStyle};
pub use types::{RichText, RichTextSegment};
