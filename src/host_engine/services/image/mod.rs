//! 图片渲染服务。
//!
//! 只负责图片层的编码与输出。
//! 不操作 Canvas，不直接写 stdout（通过 TerminalService）。
//! 上层通过 `DrawImageParams` 声明绘图意图。

mod encoders;
mod error;
mod request;
mod service;
mod sizing;

pub use error::ImageError;
pub use request::{DrawImageParams, ImageCellRect, ImageFit};
pub use service::ImageService;
pub use sizing::{CellPixelSize, ImagePixelSize, current_cell_pixel_size, pixel_size_for_rect};
