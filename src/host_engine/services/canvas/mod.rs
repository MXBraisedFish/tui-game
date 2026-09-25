pub(crate) mod buffer;
mod service;
mod surface;
mod top_layer;

pub use tg_core_style::CanvasCell;

pub use service::CanvasService;
pub use surface::{
  ResolvedScrollBoxLayout, ScrollBoxFrame, ScrollBoxId, ScrollbarSide, ScrollbarStyle, SliceFrame,
  SliceId, SurfaceFrame, SurfaceId,
};
pub(crate) use service::{PreparedScrollBox, PreparedSlice, PreparedSurface};
