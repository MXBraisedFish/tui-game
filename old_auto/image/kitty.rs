use std::io;

use base64::Engine;
use image::DynamicImage;

use super::ImageEncoder;
use super::pixel_size_for_cells;

pub(super) struct KittyEncoder;

impl ImageEncoder for KittyEncoder {
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

    // 编码为 PNG 字节
    let mut png_bytes = Vec::new();
    scaled
      .write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
      )
      .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

    // Kitty graphics protocol: f=100 is PNG, s/v are transfer pixels,
    // c/r are display columns/rows.
    let seq = format!(
      "\x1b_Ga=T,f=100,s={},v={},c={},r={},C=1;{}\x1b\\",
      pixel_w, pixel_h, w, h, b64
    );

    Ok((seq, w, h))
  }
}
