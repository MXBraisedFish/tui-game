use std::io;

use image::DynamicImage;

use super::ImageEncoder;

pub(super) struct FallbackEncoder;

impl ImageEncoder for FallbackEncoder {
  /// 使用 Unicode 半块字符 `▀`（U+2580）渲染图片。
  /// 上半格用前景色，下半格用背景色，每个字符格表示 2 个像素行。
  fn encode(
    img: &DynamicImage,
    x: u16,
    y: u16,
    max_width: u16,
    max_height: u16,
  ) -> io::Result<(String, u16, u16)> {
    let cell_w = max_width.max(1) as u32;
    // 每个字符格覆盖 2 像素行
    let char_h = max_height.max(1);
    let pixel_h = char_h as u32 * 2;

    let scaled = img.resize_exact(cell_w, pixel_h, image::imageops::FilterType::Lanczos3);
    let rgba = scaled.to_rgba8();

    let mut result = String::new();
    for cy in 0..char_h as u32 {
      result.push_str(&format!(
        "\x1b[{};{}H",
        y.saturating_add(cy as u16).saturating_add(1),
        x.saturating_add(1)
      ));
      for cx in 0..cell_w {
        let top_y = cy * 2;
        let bot_y = top_y + 1;

        let top = rgba.get_pixel(cx, top_y);
        let bot = if bot_y < pixel_h {
          rgba.get_pixel(cx, bot_y)
        } else {
          top
        };

        // 上半格 = 前景色(top)，下半格 = 背景色(bot)
        result.push_str(&format!(
          "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m\u{2580}",
          top[0], top[1], top[2], bot[0], bot[1], bot[2],
        ));
      }
      result.push_str("\x1b[0m");
    }
    result.push_str("\x1b[H");

    Ok((result, max_width.max(1), char_h))
  }
}
