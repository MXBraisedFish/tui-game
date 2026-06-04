mod buffer;
mod cell;
mod color_ansi256;
mod dirty;
mod present_diff;
mod present_full;
mod present_style;
mod rich_text;
mod service;
mod style;
mod terminal_style;
mod text;

pub use buffer::CanvasBuffer;
pub use cell::{CanvasCell, CanvasCellContent};
pub use color_ansi256::rgb_to_ansi256;
pub use dirty::DirtySpan;
pub use present_diff::present_buffer_diff;
pub use present_full::present_buffer;
pub use present_style::{apply_canvas_style, reset_canvas_style};
pub use rich_text::write_rich_text;
pub use service::CanvasService;
pub use style::CanvasStyle;
pub use terminal_style::{
  style_attributes, terminal_color_to_crossterm_color, text_color_to_crossterm_color,
};
pub use text::write_text;
