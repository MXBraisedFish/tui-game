use std::io;

use icy_sixel::SixelImage;
use image::DynamicImage;

use super::ImageEncoder;
use super::pixel_size_for_cells;

pub(super) struct SixelEncoder;

impl ImageEncoder for SixelEncoder {
  fn encode(
    img: &DynamicImage,
    _x: u16,
    _y: u16,
    max_width: u16,
    max_height: u16,
  ) -> io::Result<(String, u16, u16)> {
    let w = max_width.max(1);
    let h = max_height.max(1);
    let (pixel_w, pixel_h) = pixel_size_for_cells(w, h);

    let scaled = img.resize_exact(pixel_w, pixel_h, image::imageops::FilterType::Lanczos3);

    let rgba = scaled.to_rgba8();
    let raw = rgba.into_vec();

    let sixel_img = SixelImage::from_rgba(raw, pixel_w as usize, pixel_h as usize);
    let seq = sixel_img
      .encode()
      .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("sixel encode: {}", e)))?;

    Ok((seq, w, h))
  }
}
