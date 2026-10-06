//! Text, fill, and border drawing routed through clipped canvas surfaces.
//!
//! # Examples
//!
//! ```rust
//! use tg_service_canvas::CanvasService;
//! use tg_service_render::{BorderStyle, RenderService};
//! use tg_service_text_layout::DrawTextParams;
//!
//! fn main() {
//!   let mut canvas = CanvasService::new();
//!   canvas.resize(20, 6);
//!   let mut render = RenderService::new();
//!
//!   render.draw_border_rect(
//!     &mut canvas,
//!     0,
//!     0,
//!     6,
//!     3,
//!     &BorderStyle::Line,
//!     None,
//!     None,
//!     None,
//!     None,
//!   );
//!   render.draw_text(
//!     &mut canvas,
//!     &DrawTextParams {
//!       x: 8,
//!       y: 1,
//!       text: "hi".to_string(),
//!       ..Default::default()
//!     },
//!   );
//!
//!   let text_at = |x, y| canvas.cell_at(x, y).map(|cell| cell.text.clone());
//!   assert_eq!(text_at(0, 0).as_deref(), Some("\u{250c}"));
//!   assert_eq!(text_at(5, 2).as_deref(), Some("\u{2518}"));
//!   assert_eq!(text_at(8, 1).as_deref(), Some("h"));
//!   println!("render ok: border corners and text drawn");
//! }
//! ```

mod border;
mod service;

pub use border::{BorderCharacter, BorderStyle, CustomBorder};
pub use service::RenderService;
