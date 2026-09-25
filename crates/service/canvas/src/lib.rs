//! Canvas service: base/host/top layers, per-surface buffers (slices, scroll boxes) and text drawing.

pub mod buffer;
mod service;
mod surface;
mod top_layer;

pub use tg_core_style::CanvasCell;

pub use service::CanvasService;
pub use surface::{
  ResolvedScrollBoxLayout, ScrollBoxFrame, ScrollBoxId, ScrollbarSide, ScrollbarStyle, SliceFrame,
  SliceId, SurfaceFrame, SurfaceId,
};
pub use service::{PreparedScrollBox, PreparedSlice, PreparedSurface};
