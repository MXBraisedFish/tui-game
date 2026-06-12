use std::io;

use icy_sixel::SixelImage;
use image::DynamicImage;

use super::super::request::ImageCellRect;
use super::super::sizing::{CellPixelSize, pixel_size_for_rect};
use super::ImageEncoder;

/// Sixel 图形协议编码器。
pub struct SixelEncoder;

impl ImageEncoder for SixelEncoder {
  fn encode(image: &DynamicImage, rect: ImageCellRect, cell: CellPixelSize) -> io::Result<String> {
    let pixel = pixel_size_for_rect(rect, cell);

    let scaled = image.resize_exact(
      pixel.width,
      pixel.height,
      image::imageops::FilterType::Lanczos3,
    );

    let rgba = scaled.to_rgba8();
    let raw = rgba.into_vec();

    let sixel_img = SixelImage::from_rgba(raw, pixel.width as usize, pixel.height as usize);
    let seq = sixel_img
      .encode()
      .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("sixel encode: {e}")))?;

    Ok(seq)
  }
}
