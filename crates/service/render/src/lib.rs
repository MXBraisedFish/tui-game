//! Render service: border drawing styles and text rendering helpers on top of the canvas.

mod border;
mod service;

pub use border::{BorderCharacter, BorderStyle, CustomBorder};
pub use service::RenderService;
