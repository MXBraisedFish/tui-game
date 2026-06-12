use std::path::PathBuf;

use crate::host_engine::services::{ImageProtocol, LayoutService, TerminalService};

use super::encoders::{ImageEncoder, ITerm2Encoder, KittyEncoder, SixelEncoder};
use super::error::ImageError;
use super::request::{DrawImageParams, ImageCellRect};
use super::sizing::{CellPixelSize, current_cell_pixel_size, resolve_image_rect};

/// 图片请求签名，用于帧间差异比较。
/// 含 `cell` 是因为 resize/DPI/字体变化时 rect 可能相同但像素尺寸已变。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ImageSignature {
  protocol: ImageProtocol,
  path: PathBuf,
  rect: ImageCellRect,
  cell: CellPixelSize,
  preserve_aspect_ratio: bool,
}

/// 图片渲染服务。
///
/// 只负责图片层的编码与输出。
/// 变化时重新编码，每帧均可重放已缓存的转义序列。
/// 当前版本只支持每帧一个图片请求。
/// TODO: 多图支持时替换 `pending` 为 `Vec<DrawImageParams>`。
pub struct ImageService {
  protocol: ImageProtocol,
  pending: Option<DrawImageParams>,
  previous_signature: Option<ImageSignature>,
  current_signature: Option<ImageSignature>,
  /// 上次编码对应的签名
  cached_signature: Option<ImageSignature>,
  /// 已缓存的转义序列（变化时重新编码，每帧可重放）
  cached_sequence: Option<String>,
  force_redraw: bool,
}

impl ImageService {
  pub fn new(protocol: ImageProtocol) -> Self {
    Self {
      protocol,
      pending: None,
      previous_signature: None,
      current_signature: None,
      cached_signature: None,
      cached_sequence: None,
      force_redraw: true,
    }
  }

  /// 获取当前使用的协议。
  pub fn protocol(&self) -> ImageProtocol {
    self.protocol
  }

  /// 更新图片协议。
  pub fn set_protocol(&mut self, protocol: ImageProtocol) {
    self.protocol = protocol;
  }

  /// 帧开始，重置本帧临时状态。
  pub fn begin_frame(&mut self) {
    self.pending = None;
    self.current_signature = None;
  }

  /// 提交图片请求。
  ///
  /// 当前版本只支持每帧一个请求。
  pub fn draw(&mut self, params: DrawImageParams) -> Result<(), ImageError> {
    if self.pending.is_some() {
      return Err(ImageError::MultipleImagesUnsupported);
    }
    self.pending = Some(params);
    Ok(())
  }

  /// 标记强制重绘。
  pub fn request_render(&mut self) {
    self.force_redraw = true;
  }

  /// 检查是否需要清屏（图片请求发生变化时）。
  ///
  /// 解析 pending 图片尺寸，生成 current_signature，与 previous 比较。
  /// 只负责清屏判断，不决定 present 是否输出。
  pub fn needs_terminal_clear(
    &mut self,
    layout: &LayoutService,
  ) -> Result<bool, ImageError> {
    let Some(ref pending) = self.pending else {
      self.current_signature = None;
      return Ok(false);
    };

    let cell = current_cell_pixel_size();
    let img = load_image(&pending.path)?;
    let terminal_size = layout.get_terminal_size();

    let rect = if pending.preserve_aspect_ratio {
      resolve_image_rect(
        img.width(),
        img.height(),
        pending.x,
        pending.y,
        &pending.fit,
        cell,
        terminal_size.width,
        terminal_size.height,
      )?
    } else {
      match pending.fit {
        super::request::ImageFit::Exact { width, height } => ImageCellRect {
          x: pending.x,
          y: pending.y,
          width: width.max(1),
          height: height.max(1),
        },
        _ => resolve_image_rect(
          img.width(),
          img.height(),
          pending.x,
          pending.y,
          &pending.fit,
          cell,
          terminal_size.width,
          terminal_size.height,
        )?,
      }
    };

    let sig = ImageSignature {
      protocol: self.protocol,
      path: pending.path.clone(),
      rect,
      cell,
      preserve_aspect_ratio: pending.preserve_aspect_ratio,
    };

    let changed = self.previous_signature.as_ref() != Some(&sig);
    self.current_signature = Some(sig);

    Ok(changed || self.force_redraw)
  }

  /// 输出本帧图片（在 Canvas present 之后调用）。
  ///
  /// 每帧只要有 current_signature 就输出 cached_sequence。
  /// 签名变化或 force_redraw 时重新编码并更新缓存。
  /// 不变化 → 不重新编码，但仍重放已缓存序列。
  pub fn present(
    &mut self,
    terminal: &mut TerminalService,
    _layout: &LayoutService,
  ) -> Result<(), ImageError> {
    let Some(sig) = self.current_signature.clone() else {
      self.previous_signature = None;
      self.force_redraw = false;
      return Ok(());
    };

    let need_encode =
      self.cached_signature.as_ref() != Some(&sig) || self.cached_sequence.is_none();

    if need_encode {
      let img = load_image(&sig.path)?;
      let seq = encode_with_protocol(self.protocol, &img, sig.rect, sig.cell)?;
      self.cached_signature = Some(sig.clone());
      self.cached_sequence = Some(seq);
    }

    let seq = self
      .cached_sequence
      .as_ref()
      .ok_or(ImageError::UnsupportedProtocol)?;

    let stdout = terminal
      .writer_mut()
      .ok_or(ImageError::MissingTerminalWriter)?;

    use crossterm::QueueableCommand;
    use crossterm::cursor::MoveTo;
    use std::io::Write;

    stdout.queue(MoveTo(sig.rect.x, sig.rect.y))?;
    write!(stdout, "{seq}")?;
    stdout.queue(MoveTo(0, 0))?;
    stdout.flush()?;

    self.previous_signature = Some(sig);
    self.force_redraw = false;

    Ok(())
  }
}

// ── 内部函数 ──

fn load_image(path: &std::path::Path) -> Result<image::DynamicImage, ImageError> {
  ensure_supported_extension(path)?;
  image::open(path).map_err(|e| ImageError::Decode(e.to_string()))
}

fn ensure_supported_extension(path: &std::path::Path) -> Result<(), ImageError> {
  let ext = path
    .extension()
    .and_then(|ext| ext.to_str())
    .map(|ext| ext.to_ascii_lowercase());

  match ext.as_deref() {
    Some("png" | "jpg" | "jpeg") => Ok(()),
    _ => Err(ImageError::UnsupportedExtension),
  }
}

fn encode_with_protocol(
  protocol: ImageProtocol,
  image: &image::DynamicImage,
  rect: ImageCellRect,
  cell: CellPixelSize,
) -> Result<String, ImageError> {
  match protocol {
    ImageProtocol::Kitty => {
      KittyEncoder::encode(image, rect, cell).map_err(|e| ImageError::Encode(e.to_string()))
    }
    ImageProtocol::Sixel => {
      SixelEncoder::encode(image, rect, cell).map_err(|e| ImageError::Encode(e.to_string()))
    }
    ImageProtocol::ITerm2 => {
      ITerm2Encoder::encode(image, rect, cell).map_err(|e| ImageError::Encode(e.to_string()))
    }
    ImageProtocol::None => Err(ImageError::UnsupportedProtocol),
  }
}
