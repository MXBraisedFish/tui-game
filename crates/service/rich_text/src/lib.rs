//! Plain and tagged text parsing with explicit formatting parameters.
//!
//! # Examples
//!
//! ```rust
//! use tg_service_rich_text::{RichTextService, TextMode};
//!
//! fn main() {
//!   let service = RichTextService::new();
//!   let rich = service.parse_mode("plain <b>bold</b>", None, TextMode::Rich);
//!   let bold = rich
//!     .segments
//!     .iter()
//!     .find(|segment| segment.text == "bold")
//!     .expect("bold segment");
//!   assert!(bold.style.bold);
//!   println!("rich_text ok: {} segments", rich.segments.len());
//! }
//! ```

mod params;
mod parser;
mod service;

pub use tg_core_style::parse_text_color;

pub use params::RichTextParams;

pub use service::{RichTextService, TextMode};
pub use tg_core_style::{TerminalColor, TextColor, TextStyle};

pub use tg_core_style::{RichText, RichTextSegment};
