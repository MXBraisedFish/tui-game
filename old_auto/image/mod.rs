mod fallback;
mod iterm2;
mod kitty;
mod sixel;

use std::io;
use std::path::Path;

use crossterm::{QueueableCommand, cursor::MoveTo};
use image::DynamicImage;

use super::terminal::TerminalService;
use super::terminal_capabilities::ImageProtocol;

/// 图片编码器特征。每种终端图片协议实现此特征。
trait ImageEncoder {
  /// 将图片编码为终端转义序列。
  /// 返回 `(转义序列, 渲染宽度_字符格, 渲染高度_字符格)`。
  fn encode(
    img: &DynamicImage,
    x: u16,
    y: u16,
    max_width: u16,
    max_height: u16,
  ) -> io::Result<(String, u16, u16)>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageSize {
  Width(u16),
  Height(u16),
}

/// 图片渲染服务。根据终端能力选择协议，提供统一的 `render()` API。
pub struct ImageService {
  protocol: ImageProtocol,
}

impl ImageService {
  pub fn new(protocol: ImageProtocol) -> Self {
    Self { protocol }
  }

  /// 获取当前使用的协议。
  pub fn protocol(&self) -> ImageProtocol {
    self.protocol
  }

  /// 加载图片文件。
  pub fn load(&self, path: &Path) -> io::Result<DynamicImage> {
    ensure_supported_extension(path)?;
    image::open(path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
  }

  /// 编码图片为转义序列（不输出），用于测试。
  pub fn encode(
    &self,
    path: &Path,
    x: u16,
    y: u16,
    max_width: u16,
    max_height: u16,
  ) -> io::Result<(String, u16, u16)> {
    let img = self.load(path)?;
    self.encode_image(&img, x, y, max_width, max_height)
  }

  pub fn encode_path(
    &self,
    path: &Path,
    x: u16,
    y: u16,
    size: ImageSize,
  ) -> io::Result<(String, u16, u16)> {
    let img = self.load(path)?;
    let (width, height) = dimensions_for_image(&img, size);
    self.encode_image(&img, x, y, width, height)
  }

  /// 编码已加载的图片（使用服务绑定的协议）。
  pub fn encode_image(
    &self,
    img: &DynamicImage,
    x: u16,
    y: u16,
    max_width: u16,
    max_height: u16,
  ) -> io::Result<(String, u16, u16)> {
    Self::encode_with_protocol(self.protocol, img, x, y, max_width, max_height)
  }

  /// 编码图片（**强制指定协议**，忽略服务绑定）。
  /// 用于图片能力检测步骤：依次尝试四种协议，看哪种能正常显示。
  pub fn encode_with_protocol(
    protocol: ImageProtocol,
    img: &DynamicImage,
    x: u16,
    y: u16,
    max_width: u16,
    max_height: u16,
  ) -> io::Result<(String, u16, u16)> {
    match protocol {
      ImageProtocol::Kitty => kitty::KittyEncoder::encode(img, x, y, max_width, max_height),
      ImageProtocol::Sixel => sixel::SixelEncoder::encode(img, x, y, max_width, max_height),
      ImageProtocol::ITerm2 => iterm2::ITerm2Encoder::encode(img, x, y, max_width, max_height),
      ImageProtocol::None => fallback::FallbackEncoder::encode(img, x, y, max_width, max_height),
    }
  }

  /// 给定目标宽度（字符格），返回等比缩放后的 (宽, 高)。
  /// 不依赖协议，按图片像素比例和终端字符格像素比例计算。
  pub fn dimensions_for_width(&self, path: &Path, target_width: u16) -> io::Result<(u16, u16)> {
    let img = self.load(path)?;
    Ok(dimensions_for_image(&img, ImageSize::Width(target_width)))
  }

  /// 给定目标高度（字符格），返回等比缩放后的 (宽, 高)。
  /// 不依赖协议，按图片像素比例和终端字符格像素比例计算。
  pub fn dimensions_for_height(&self, path: &Path, target_height: u16) -> io::Result<(u16, u16)> {
    let img = self.load(path)?;
    Ok(dimensions_for_image(&img, ImageSize::Height(target_height)))
  }

  pub fn dimensions_for_size(&self, path: &Path, size: ImageSize) -> io::Result<(u16, u16)> {
    let img = self.load(path)?;
    Ok(dimensions_for_image(&img, size))
  }

  /// 生成一张检测用测试图（RGB 渐变方块，程序化生成，无需外部文件）。
  pub fn generate_test_image() -> DynamicImage {
    let w: u32 = 200;
    let h: u32 = 100;
    let mut img = image::RgbaImage::new(w, h);
    // 红→绿→蓝水平渐变
    for x in 0..w {
      let t = x as f32 / (w - 1) as f32;
      let (r, g, b) = if t < 0.5 {
        // 红→绿
        let t2 = t * 2.0;
        ((255.0 * (1.0 - t2)) as u8, (255.0 * t2) as u8, 0u8)
      } else {
        // 绿→蓝
        let t2 = (t - 0.5) * 2.0;
        (0u8, (255.0 * (1.0 - t2)) as u8, (255.0 * t2) as u8)
      };
      for y in 0..h {
        img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
      }
    }
    DynamicImage::ImageRgba8(img)
  }

  /// 在终端 (x, y) 处渲染图片。
  /// 直接向 stdout 写入转义序列，应在 `canvas.present()` 之后调用。
  pub fn render(
    &self,
    terminal: &mut TerminalService,
    img: &DynamicImage,
    x: u16,
    y: u16,
    max_width: u16,
    max_height: u16,
  ) -> io::Result<(u16, u16)> {
    let (seq, w, h) = self.encode_image(img, x, y, max_width, max_height)?;
    if let Some(stdout) = terminal.writer_mut() {
      stdout.queue(MoveTo(x, y))?;
      // 直接写入原始转义序列（不经过 crossterm 的 command 系统）
      use std::io::Write;
      write!(stdout, "{}", seq)?;
      stdout.queue(MoveTo(0, 0))?;
      stdout.flush()?;
    }
    Ok((w, h))
  }

  pub fn render_path(
    &self,
    terminal: &mut TerminalService,
    path: &Path,
    x: u16,
    y: u16,
    size: ImageSize,
  ) -> io::Result<(u16, u16)> {
    let (seq, w, h) = self.encode_path(path, x, y, size)?;
    if let Some(stdout) = terminal.writer_mut() {
      stdout.queue(MoveTo(x, y))?;
      use std::io::Write;
      write!(stdout, "{}", seq)?;
      stdout.queue(MoveTo(0, 0))?;
      stdout.flush()?;
    }
    Ok((w, h))
  }
}

fn ensure_supported_extension(path: &Path) -> io::Result<()> {
  let ext = path
    .extension()
    .and_then(|ext| ext.to_str())
    .map(|ext| ext.to_ascii_lowercase());

  match ext.as_deref() {
    Some("png" | "jpg" | "jpeg") => Ok(()),
    _ => Err(io::Error::new(
      io::ErrorKind::InvalidInput,
      "only png, jpg and jpeg images are supported",
    )),
  }
}

fn dimensions_for_image(img: &DynamicImage, size: ImageSize) -> (u16, u16) {
  dimensions_for_image_with_cell(img, size, cell_pixel_size())
}

fn dimensions_for_image_with_cell(
  img: &DynamicImage,
  size: ImageSize,
  cell_size: (u32, u32),
) -> (u16, u16) {
  let pw = img.width() as f64;
  let ph = img.height() as f64;
  if pw == 0.0 || ph == 0.0 {
    return (0, 0);
  }
  let (cell_w, cell_h) = cell_size;
  let cell_w = cell_w.max(1) as f64;
  let cell_h = cell_h.max(1) as f64;

  match size {
    ImageSize::Width(width) => {
      let height = (width as f64 * ph / pw * cell_w / cell_h).round() as u16;
      (width, height.max(1))
    }
    ImageSize::Height(height) => {
      let width = (height as f64 * pw / ph * cell_h / cell_w).round() as u16;
      (width.max(1), height)
    }
  }
}

pub(super) fn cell_pixel_size() -> (u32, u32) {
  let Ok(size) = crossterm::terminal::window_size() else {
    return (8, 16);
  };
  if size.columns == 0 || size.rows == 0 || size.width == 0 || size.height == 0 {
    return (8, 16);
  }
  let cell_w = (size.width as u32 / size.columns as u32).max(1);
  let cell_h = (size.height as u32 / size.rows as u32).max(1);
  (cell_w, cell_h)
}

pub(super) fn pixel_size_for_cells(width: u16, height: u16) -> (u32, u32) {
  let (cell_w, cell_h) = cell_pixel_size();
  (
    (width as u32).saturating_mul(cell_w).max(1),
    (height as u32).saturating_mul(cell_h).max(1),
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use base64::Engine;
  use std::path::Path;

  #[test]
  fn kitty_uses_png_format_and_cell_placement() {
    let img = ImageService::generate_test_image();
    let (seq, w, h) = ImageService::encode_with_protocol(ImageProtocol::Kitty, &img, 0, 0, 12, 6)
      .expect("kitty encode");

    assert_eq!((w, h), (12, 6));
    assert!(seq.contains("f=100"));
    assert!(seq.contains("C=1"));
    assert!(seq.contains("c=12"));
    assert!(seq.contains("r=6"));
  }

  #[test]
  fn iterm2_uses_cell_units_with_size_and_clear_area() {
    let img = ImageService::generate_test_image();
    let (seq, w, h) = ImageService::encode_with_protocol(ImageProtocol::ITerm2, &img, 0, 0, 12, 6)
      .expect("iterm2 encode");

    assert_eq!((w, h), (12, 6));
    assert!(seq.contains("inline=1"));
    assert!(seq.contains("size="));
    assert!(seq.contains("width=12;height=6"));
    assert!(seq.contains("doNotMoveCursor=1"));
    assert!(seq.starts_with("\x1b[12X\x1b[1B"));
    assert!(seq.contains("\x1b[6A\x1b]1337;File="));
    assert!(!seq.contains("px"));
    assert!(!seq.contains("preserveAspectRatio=1"));
  }

  #[test]
  fn iterm2_embeds_rgba_png_data() {
    let img = ImageService::generate_test_image();
    let (seq, _, _) = ImageService::encode_with_protocol(ImageProtocol::ITerm2, &img, 0, 0, 12, 6)
      .expect("iterm2 encode");
    let b64 = seq
      .split_once("doNotMoveCursor=1:")
      .map(|(_, payload)| payload)
      .and_then(|payload| payload.strip_suffix('\x07'))
      .filter(|_| seq.contains("\x1b]1337;File=inline=1;size="))
      .expect("iterm2 payload");
    let png = base64::engine::general_purpose::STANDARD
      .decode(b64)
      .expect("base64 png");
    let decoded = image::load_from_memory(&png).expect("png decode");

    assert!(matches!(decoded, DynamicImage::ImageRgba8(_)));
  }

  #[test]
  fn sixel_encoder_returns_single_complete_dcs_sequence() {
    let img = ImageService::generate_test_image();
    let (seq, w, h) = ImageService::encode_with_protocol(ImageProtocol::Sixel, &img, 0, 0, 12, 6)
      .expect("sixel encode");

    assert_eq!((w, h), (12, 6));
    assert!(seq.starts_with("\x1bP9;1;0q"));
    assert_eq!(seq.matches("\x1bP").count(), 1);
    assert!(!seq.starts_with("\x1bPq\x1bP"));
  }

  #[test]
  fn fallback_positions_each_rendered_row() {
    let img = ImageService::generate_test_image();
    let (seq, w, h) = ImageService::encode_with_protocol(ImageProtocol::None, &img, 3, 4, 5, 2)
      .expect("fallback encode");

    assert_eq!((w, h), (5, 2));
    assert!(seq.contains("\x1b[5;4H"));
    assert!(seq.contains("\x1b[6;4H"));
    assert!(seq.ends_with("\x1b[H"));
  }

  #[test]
  fn load_rejects_unsupported_extension() {
    let service = ImageService::new(ImageProtocol::None);
    let err = service
      .load(Path::new("assets/images/test/test.gif"))
      .unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
  }

  #[test]
  fn square_image_width_uses_cell_aspect_ratio() {
    let img = DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
    let size = dimensions_for_image_with_cell(&img, ImageSize::Width(20), (8, 16));
    assert_eq!(size, (20, 10));
  }

  #[test]
  fn wide_and_tall_images_keep_visual_ratio() {
    let wide = DynamicImage::ImageRgba8(image::RgbaImage::new(200, 100));
    let tall = DynamicImage::ImageRgba8(image::RgbaImage::new(100, 200));

    assert_eq!(
      dimensions_for_image_with_cell(&wide, ImageSize::Width(20), (8, 16)),
      (20, 5)
    );
    assert_eq!(
      dimensions_for_image_with_cell(&tall, ImageSize::Height(10), (8, 16)),
      (10, 10)
    );
  }
}
