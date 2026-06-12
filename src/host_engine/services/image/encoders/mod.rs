mod kitty;
mod sixel;
mod iterm2;

pub use kitty::KittyEncoder;
pub use sixel::SixelEncoder;
pub use iterm2::ITerm2Encoder;

use std::io;
use image::DynamicImage;

use super::request::ImageCellRect;
use super::sizing::CellPixelSize;

/// 图片编码器特征。
///
/// 每种终端图片协议实现此特征。
/// Encoder 只返回 ANSI/OSC/Sixel 字符串，不执行 MoveTo/Clear/flush。
pub trait ImageEncoder {
  /// 将图片编码为终端转义序列。
  ///
  /// 返回纯转义序列字符串，调用方负责在正确位置输出。
  fn encode(
    image: &DynamicImage,
    rect: ImageCellRect,
    cell: CellPixelSize,
  ) -> io::Result<String>;
}
