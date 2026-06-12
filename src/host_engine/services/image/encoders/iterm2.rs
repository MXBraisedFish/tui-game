use std::fmt::Write as FmtWrite;
use std::io;

use base64::Engine;
use image::DynamicImage;

use super::super::request::ImageCellRect;
use super::super::sizing::{CellPixelSize, pixel_size_for_rect};
use super::ImageEncoder;

/// iTerm2 inline 图片协议编码器。
///
/// 只做 best-effort，不承诺 100% 稳定。
/// 自带局部 `clear_area()`，不移动光标，不 flush。
pub struct ITerm2Encoder;

impl ImageEncoder for ITerm2Encoder {
  fn encode(image: &DynamicImage, rect: ImageCellRect, cell: CellPixelSize) -> io::Result<String> {
    let pixel = pixel_size_for_rect(rect, cell);

    let scaled = image
      .resize_exact(
        pixel.width,
        pixel.height,
        image::imageops::FilterType::Lanczos3,
      )
      .to_rgba8();

    let mut png_bytes = Vec::new();
    DynamicImage::ImageRgba8(scaled)
      .write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
      )
      .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

    let mut seq = String::new();
    clear_area(&mut seq, rect.width, rect.height);

    let _ = write!(
      seq,
      "\x1b]1337;File=inline=1;size={};width={}px;height={}px;doNotMoveCursor=1:",
      png_bytes.len(),
      pixel.width,
      pixel.height,
    );
    seq.push_str(&b64);
    seq.push('\x07');

    Ok(seq)
  }
}

fn clear_area(seq: &mut String, width: u16, height: u16) {
  for _ in 0..height {
    let _ = write!(seq, "\x1b[{}X", width);
    let _ = write!(seq, "\x1b[1B");
  }

  let _ = write!(seq, "\x1b[{}A", height);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn iterm2_uses_px_size_and_clear_area_prelude() {
    let image = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
      2,
      2,
      image::Rgba([0, 0, 255, 255]),
    ));
    let seq = ITerm2Encoder::encode(
      &image,
      ImageCellRect {
        x: 0,
        y: 0,
        width: 3,
        height: 2,
      },
      CellPixelSize {
        width: 8,
        height: 16,
      },
    )
    .expect("encode iterm2");

    assert!(seq.starts_with("\x1b[3X\x1b[1B\x1b[3X\x1b[1B\x1b[2A"));
    assert!(seq.contains("inline=1"));
    assert!(seq.contains("width=24px;height=32px"));
    assert!(seq.contains("doNotMoveCursor=1"));
    assert!(!seq.contains("preserveAspectRatio=0"));
    assert!(!seq.contains("width=3;height=2"));
  }
}
