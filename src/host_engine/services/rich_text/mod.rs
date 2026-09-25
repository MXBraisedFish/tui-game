mod params;
mod parser;
mod service;

pub use tg_core_style::parse_text_color;

pub use params::RichTextParams;

pub use service::{RichTextService, TextMode};
pub use tg_core_style::{TerminalColor, TextColor, TextStyle};

pub use tg_core_style::{RichText, RichTextSegment};
