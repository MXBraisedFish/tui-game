//! Grapheme segmentation and display widths measured in terminal columns.
//!
//! # Examples
//!
//! ```rust
//! use tg_core_unicode::{display_width, graphemes};
//!
//! assert_eq!(display_width("abc"), 3);
//! let combined = graphemes("e\u{0301}");
//! assert_eq!(combined.len(), 1);
//! assert_eq!(combined[0].display_width, 1);
//! ```

mod measure;
mod types;

pub use measure::{char_width, display_width, graphemes};
pub use types::GraphemeInfo;
