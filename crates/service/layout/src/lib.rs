//! Terminal-cell measurements and coordinates within physical and developer viewports.
//!
//! # Examples
//!
//! ```rust
//! use tg_core_geometry::Rect;
//! use tg_service_layout::LayoutService;
//!
//! fn main() {
//!   let mut layout = LayoutService::new();
//!   layout.resize_physical(100, 40);
//!   layout.set_developer_viewport(Rect {
//!     x: 10,
//!     y: 5,
//!     width: 200,
//!     height: 20,
//!   });
//!   assert_eq!(
//!     layout.developer_width(),
//!     90,
//!     "viewport is clipped to the physical width"
//!   );
//!   let x = layout.resolve_x(LayoutService::ALIGN_CENTER, 10, 0);
//!   assert_eq!(x, 40);
//!   println!(
//!     "layout ok: viewport {:?}, centered x {x}",
//!     layout.developer_viewport_rect()
//!   );
//! }
//! ```

mod measure;
mod position;
mod service;
use tg_core_geometry as types;

pub use service::LayoutService;
pub use types::{Rect, Size};
