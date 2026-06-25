
mod color;
mod params;
mod parser;
mod service;
mod style;
mod types;

pub use color::parse_text_color;

pub use params::RichTextParams;

pub use service::RichTextService;
pub use style::{TerminalColor, TextColor, TextStyle};

pub use types::{RichText, RichTextSegment};
