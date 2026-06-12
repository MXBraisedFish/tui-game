use super::error::ImageError;
use super::request::{ImageCellRect, ImageFit};

/// 单个终端字符格的像素尺寸。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellPixelSize {
  pub width: u32,
  pub height: u32,
}

/// 图片渲染目标像素尺寸。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImagePixelSize {
  pub width: u32,
  pub height: u32,
}

/// 查询当前终端字符格的像素尺寸。
///
/// 通过 `crossterm::terminal::window_size()` 计算。
/// 如果无法获取，回退到默认 8×16。
pub fn current_cell_pixel_size() -> CellPixelSize {
  match crossterm::terminal::window_size() {
    Ok(size) if size.columns > 0 && size.rows > 0 && size.width > 0 && size.height > 0 => {
      CellPixelSize {
        width: (size.width as u32 / size.columns as u32).max(1),
        height: (size.height as u32 / size.rows as u32).max(1),
      }
    }
    _ => CellPixelSize {
      width: 8,
      height: 16,
    },
  }
}

/// 将字符格矩形换算为像素尺寸。
pub fn pixel_size_for_rect(rect: ImageCellRect, cell: CellPixelSize) -> ImagePixelSize {
  ImagePixelSize {
    width: rect.width as u32 * cell.width,
    height: rect.height as u32 * cell.height,
  }
}

/// 根据图片原始像素尺寸和缩放策略，计算图片在终端中的字符格区域。
///
/// 所有计算在字符格单位下进行，等比缩放时考虑 cell 像素宽高比。
pub fn resolve_image_rect(
  image_width_px: u32,
  image_height_px: u32,
  x: u16,
  y: u16,
  fit: &ImageFit,
  cell: CellPixelSize,
  terminal_width: u16,
  terminal_height: u16,
) -> Result<ImageCellRect, ImageError> {
  let pw = image_width_px as f64;
  let ph = image_height_px as f64;

  let cell_w = cell.width.max(1) as f64;
  let cell_h = cell.height.max(1) as f64;

  let (width, height): (u16, u16) = match fit {
    ImageFit::Width(w) => {
      let w = (*w).max(1) as f64;
      let h = (w * ph / pw * cell_w / cell_h).round();
      (w as u16, (h as u16).max(1))
    }
    ImageFit::Height(h) => {
      let h = (*h).max(1) as f64;
      let w = (h * pw / ph * cell_h / cell_w).round();
      ((w as u16).max(1), h as u16)
    }
    ImageFit::Exact { width, height } => ((*width).max(1), (*height).max(1)),
    ImageFit::Original => {
      let w = (pw / cell_w).round() as u16;
      let h = (ph / cell_h).round() as u16;
      (w.max(1), h.max(1))
    }
  };

  // clamp 到终端可视区域
  let max_w = terminal_width.saturating_sub(x);
  let max_h = terminal_height.saturating_sub(y).saturating_sub(1); // 底部留 1 行防滚屏
  let width = width.min(max_w);
  let height = height.min(max_h);

  if width == 0 || height == 0 {
    return Err(ImageError::OutOfBounds);
  }

  Ok(ImageCellRect {
    x,
    y,
    width,
    height,
  })
}
