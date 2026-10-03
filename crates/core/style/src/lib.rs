//! Terminal colors, text styles, wide-cell markers, and composed frame data.
//!
//! # Examples
//!
//! ```rust
//! use tg_core_style::{CanvasCell, ComposedCell, ComposedFrame, TextStyle};
//!
//! let mut style = TextStyle::default();
//! assert!(style.enable_style("bold"));
//! let mut frame = ComposedFrame::new(2, 1);
//! frame.set(0, 0, ComposedCell::Text(CanvasCell::styled("A", style)));
//! assert!(matches!(frame.get(0, 0), Some(ComposedCell::Text(_))));
//! assert!(frame.get(2, 0).is_none());
//! ```

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
