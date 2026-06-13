use std::path::PathBuf;

use crate::host_engine::services::{ImageProtocol, LayoutService};

use super::encoders::{ITerm2Encoder, ImageEncoder, KittyEncoder, SixelEncoder};
use super::error::ImageError;
use super::request::{DrawImageParams, ImageCellRect};
use super::sizing::{CellPixelSize, current_cell_pixel_size, resolve_image_rect};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageLayerFrame {
  pub images: Vec<LayerImage>,
  pub dirty: bool,
  pub removed_regions: Vec<ImageCellRect>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerImage {
  pub id: u64,
  pub protocol: ImageProtocol,
  pub rect: ImageCellRect,
  pub signature: ImageSignature,
  pub sequence: String,
}

/// 图片请求签名，用于帧间差异比较。
/// 含 `cell` 是因为 resize/DPI/字体变化时 rect 可能相同但像素尺寸已变。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageSignature {
  pub protocol: ImageProtocol,
  pub path: PathBuf,
  pub rect: ImageCellRect,
  pub cell: CellPixelSize,
  pub preserve_aspect_ratio: bool,
}

/// 图片渲染服务。
///
/// 只负责图片层请求、尺寸解析与协议编码缓存。
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

  pub fn build_layer(&mut self, layout: &LayoutService) -> Result<ImageLayerFrame, ImageError> {
    let Some(ref pending) = self.pending else {
      self.current_signature = None;
      return Ok(ImageLayerFrame {
        images: Vec::new(),
        dirty: self.previous_signature.is_some(),
        removed_regions: previous_rects(&self.previous_signature),
      });
    };

    if self.protocol == ImageProtocol::None {
      self.current_signature = None;
      return Ok(ImageLayerFrame {
        images: Vec::new(),
        dirty: self.previous_signature.is_some(),
        removed_regions: previous_rects(&self.previous_signature),
      });
    }

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
    self.current_signature = Some(sig.clone());

    let sequence = self.sequence_for_signature(&sig, &img)?;

    let removed_regions = match &self.previous_signature {
      Some(previous) if previous.rect != rect => vec![previous.rect],
      Some(previous) if previous.protocol != self.protocol => vec![previous.rect],
      _ => Vec::new(),
    };

    Ok(ImageLayerFrame {
      images: vec![LayerImage {
        id: 1,
        protocol: self.protocol,
        rect,
        signature: sig,
        sequence,
      }],
      dirty: changed || self.force_redraw,
      removed_regions,
    })
  }

  pub fn end_frame(&mut self) {
    self.previous_signature = self.current_signature.clone();
    self.force_redraw = false;

    if self.current_signature.is_none() {
      self.cached_signature = None;
      self.cached_sequence = None;
    }
  }

  fn sequence_for_signature(
    &mut self,
    sig: &ImageSignature,
    image: &image::DynamicImage,
  ) -> Result<String, ImageError> {
    let need_encode = self.cached_signature.as_ref() != Some(sig) || self.cached_sequence.is_none();

    if need_encode {
      let seq = encode_with_protocol(sig.protocol, image, sig.rect, sig.cell)?;
      self.cached_signature = Some(sig.clone());
      self.cached_sequence = Some(seq);
    }

    self
      .cached_sequence
      .clone()
      .ok_or(ImageError::UnsupportedProtocol)
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

fn previous_rects(signature: &Option<ImageSignature>) -> Vec<ImageCellRect> {
  signature
    .as_ref()
    .map(|sig| vec![sig.rect])
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
  use super::super::request::ImageFit;
  use super::*;

  fn write_test_png(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(name);
    let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
      2,
      2,
      image::Rgba([255, 0, 0, 255]),
    ));
    img.save(&path).expect("write test image");
    path
  }

  #[test]
  fn none_protocol_builds_empty_dirty_layer_with_removed_region() {
    let mut service = ImageService::new(ImageProtocol::None);
    service.previous_signature = Some(ImageSignature {
      protocol: ImageProtocol::Kitty,
      path: PathBuf::from("old.png"),
      rect: ImageCellRect {
        x: 1,
        y: 2,
        width: 3,
        height: 4,
      },
      cell: CellPixelSize {
        width: 8,
        height: 16,
      },
      preserve_aspect_ratio: false,
    });

    let layer = service
      .build_layer(&LayoutService::new())
      .expect("build none layer");

    assert!(layer.images.is_empty());
    assert!(layer.dirty);
    assert_eq!(
      layer.removed_regions,
      vec![ImageCellRect {
        x: 1,
        y: 2,
        width: 3,
        height: 4,
      }]
    );
    assert_eq!(service.current_signature, None);
  }

  #[test]
  fn build_layer_uses_exact_rect_for_iterm2() {
    let path = write_test_png("tui-game-image-service-iterm2.png");
    let mut service = ImageService::new(ImageProtocol::ITerm2);
    service
      .draw(DrawImageParams {
        x: 3,
        y: 4,
        path,
        fit: ImageFit::Exact {
          width: 20,
          height: 8,
        },
        preserve_aspect_ratio: false,
      })
      .expect("draw image");

    let layer = service
      .build_layer(&LayoutService::new())
      .expect("build image layer");

    assert_eq!(layer.images.len(), 1);
    assert_eq!(
      layer.images[0].rect,
      ImageCellRect {
        x: 3,
        y: 4,
        width: 20,
        height: 8,
      }
    );
    assert_eq!(layer.images[0].protocol, ImageProtocol::ITerm2);
    assert!(layer.dirty);
    assert!(!layer.images[0].sequence.is_empty());
  }

  #[test]
  fn end_frame_moves_current_signature_to_previous() {
    let mut service = ImageService::new(ImageProtocol::Kitty);
    service.current_signature = Some(ImageSignature {
      protocol: ImageProtocol::Kitty,
      path: PathBuf::from("image.png"),
      rect: ImageCellRect {
        x: 0,
        y: 0,
        width: 1,
        height: 1,
      },
      cell: CellPixelSize {
        width: 8,
        height: 16,
      },
      preserve_aspect_ratio: true,
    });
    service.force_redraw = true;

    service.end_frame();

    assert_eq!(service.previous_signature, service.current_signature);
    assert!(!service.force_redraw);
  }

  #[test]
  fn build_layer_reuses_cached_sequence_when_signature_is_unchanged() {
    let path = write_test_png("tui-game-image-service-cache.png");
    let mut service = ImageService::new(ImageProtocol::Kitty);
    let request = DrawImageParams {
      x: 1,
      y: 2,
      path,
      fit: ImageFit::Exact {
        width: 4,
        height: 3,
      },
      preserve_aspect_ratio: false,
    };

    service.draw(request.clone()).expect("draw first image");
    let first = service
      .build_layer(&LayoutService::new())
      .expect("first layer");
    service.end_frame();

    service.begin_frame();
    service.draw(request).expect("draw same image");
    let second = service
      .build_layer(&LayoutService::new())
      .expect("second layer");

    assert_eq!(first.images[0].sequence, second.images[0].sequence);
    assert!(!second.dirty);
  }
}
