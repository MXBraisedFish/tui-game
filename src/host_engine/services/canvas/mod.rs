mod buffer;
mod cell;
mod layout;
mod present;
mod rich_text;
mod service;
mod style;
mod terminal_style;
mod text;

pub use buffer::CanvasBuffer;
pub use cell::{CanvasCell, CanvasCellContent};
pub use layout::write_centered_text;
pub use present::present_buffer;
pub use rich_text::write_rich_text;
pub use service::CanvasService;
pub use style::CanvasStyle;
pub use terminal_style::{
  style_attributes, terminal_color_to_crossterm_color, text_color_to_crossterm_color,
};
pub use text::write_text;
