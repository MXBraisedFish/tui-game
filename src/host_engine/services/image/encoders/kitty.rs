use std::io;

use base64::Engine;
use image::DynamicImage;

use super::ImageEncoder;
use super::super::request::ImageCellRect;
use super::super::sizing::{CellPixelSize, pixel_size_for_rect};

/// Kitty 图形协议编码器。
///
/// 使用 PNG + base64 编码，c/r 为终端字符格尺寸，s/v 为像素尺寸，C=1 不移动光标。
pub struct KittyEncoder;

impl ImageEncoder for KittyEncoder {
  fn encode(
    image: &DynamicImage,
    rect: ImageCellRect,
    cell: CellPixelSize,
  ) -> io::Result<String> {
    let pixel = pixel_size_for_rect(rect, cell);

    let scaled = image.resize_exact(
      pixel.width,
      pixel.height,
      image::imageops::FilterType::Lanczos3,
    );

    let mut png_bytes = Vec::new();
    scaled
      .write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
      )
      .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

    // f=100 → PNG，s/v → 传输像素，c/r → 显示列/行，C=1 → 不移动光标
    let seq = format!(
      "\x1b_Ga=T,f=100,s={},v={},c={},r={},C=1;{}\x1b\\",
      pixel.width, pixel.height, rect.width, rect.height, b64
    );

    Ok(seq)
  }
}
