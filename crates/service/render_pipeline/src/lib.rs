//! Frame composition and terminal presentation with incremental cell updates.
//!
//! # Examples
//!
//! ```rust
//! use tg_core_style::{ComposedCell, TextStyle};
//! use tg_service_canvas::CanvasService;
//! use tg_service_render_pipeline::FrameCompositor;
//!
//! fn main() {
//!   let mut canvas = CanvasService::new();
//!   canvas.styled_text(1, 2, "a", TextStyle::default());
//!
//!   let frame = FrameCompositor::new().compose(&canvas);
//!
//!   assert!(matches!(
//!     frame.get(1, 2),
//!     Some(ComposedCell::Text(cell)) if cell.text == "a"
//!   ));
//!   println!("render_pipeline ok: composed cell (1, 2) = \"a\"");
//! }
//! ```

mod compositor;
mod presenter;

pub use compositor::FrameCompositor;
pub use presenter::FramePresenter;
pub use tg_core_style::{ComposedCell, ComposedFrame};
