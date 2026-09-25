//! Terminal display-width measurement: character/string column widths and grapheme splitting.

mod measure;
mod types;

pub use measure::{char_width, display_width, graphemes};
pub use types::GraphemeInfo;
