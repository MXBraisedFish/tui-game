//! Layout service: physical/developer viewport sizes, anchor resolution and text measurement.

mod measure;
mod position;
mod service;
use tg_core_geometry as types;

pub use service::LayoutService;
pub use types::{Rect, Size};
