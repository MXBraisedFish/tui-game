use std::io;

use base64::Engine;
use image::DynamicImage;

use super::ImageEncoder;
use super::super::request::ImageCellRect;
use super::super::sizing::{CellPixelSize, pixel_size_for_rect};

/// iTerm2 inline 图片协议编码器。
///
/// 只做 best-effort，不承诺 100% 稳定。
/// 不含 `clear_area()`，不移动光标。
pub struct ITerm2Encoder;

impl ImageEncoder for ITerm2Encoder {
  fn encode(
    image: &DynamicImage,
    rect: ImageCellRect,
    cell: CellPixelSize,
  ) -> io::Result<String> {
    let pixel = pixel_size_for_rect(rect, cell);

    let scaled = image
      .resize_exact(pixel.width, pixel.height, image::imageops::FilterType::Lanczos3)
      .to_rgba8();

    let mut png_bytes = Vec::new();
    DynamicImage::ImageRgba8(scaled)
      .write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
      )
      .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

    // 使用字符格单位声明宽高，避免部分终端将 px 误解释为大量单元格
    let seq = format!(
      "\x1b]1337;File=inline=1;size={};width={};height={};preserveAspectRatio=0;doNotMoveCursor=1:{}\x07",
      png_bytes.len(),
      rect.width,
      rect.height,
      b64
    );

    Ok(seq)
  }
}
