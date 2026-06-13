//! 图片层服务。
//!
//! 只负责图片请求、尺寸解析与协议编码缓存。
//! 不操作 Canvas，不直接写 stdout。
//! 上层通过 `DrawImageParams` 声明绘图意图。

mod encoders;
mod error;
mod request;
mod service;
mod sizing;

pub use request::{DrawImageParams, ImageCellRect, ImageFit};
pub use service::{ImageLayerFrame, ImageService, ImageSignature, LayerImage};
pub use sizing::CellPixelSize;
