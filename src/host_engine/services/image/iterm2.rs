use std::io;

use base64::Engine;
use image::DynamicImage;

use super::ImageEncoder;
use super::pixel_size_for_cells;

pub(super) struct ITerm2Encoder;

impl ImageEncoder for ITerm2Encoder {
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

    let scaled = img
      .resize_exact(pixel_w, pixel_h, image::imageops::FilterType::Lanczos3)
      .to_rgba8();

    let mut png_bytes = Vec::new();
    DynamicImage::ImageRgba8(scaled)
      .write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
      )
      .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

    // Keep the display size in terminal cells. Some terminals that advertise
    // iTerm2 support handle `px` sizes as large cell counts and scroll the UI.
    let seq = format!(
      "{}\x1b]1337;File=inline=1;size={};width={};height={};doNotMoveCursor=1:{}\x07",
      clear_area(w, h),
      png_bytes.len(),
      w,
      h,
      b64
    );

    Ok((seq, w, h))
  }
}

fn clear_area(width: u16, height: u16) -> String {
  if height <= 1 {
    return format!("\x1b[{}X", width);
  }

  let mut seq = String::new();
  for _ in 0..height {
    seq.push_str(&format!("\x1b[{}X\x1b[1B", width));
  }
  seq.push_str(&format!("\x1b[{}A", height));
  seq
}
