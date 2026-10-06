//! Resolution of symbolic positions against available terminal-cell bounds.

use super::types::{Position, Size};

/// The align left used by this module.
pub const ALIGN_LEFT: &str = "left";

/// The align center used by this module.
pub const ALIGN_CENTER: &str = "center";

/// The align right used by this module.
pub const ALIGN_RIGHT: &str = "right";

/// The align top used by this module.
pub const ALIGN_TOP: &str = "top";

/// The align middle used by this module.
pub const ALIGN_MIDDLE: &str = "middle";

/// The align bottom used by this module.
pub const ALIGN_BOTTOM: &str = "bottom";

/// Resolve a symbolic horizontal position within the available terminal columns.
///
/// # Arguments
///
/// * `size` - The size.
/// * `x_anchor` - The x anchor.
/// * `content_width` - The content width in terminal columns.
/// * `offset_x` - The offset x.
pub fn resolve_x(size: Size, x_anchor: &str, content_width: u16, offset_x: u16) -> u16 {
  let term_w = size.width;
  match x_anchor {
    ALIGN_LEFT => offset_x,
    ALIGN_CENTER => term_w.saturating_sub(content_width) / 2 + offset_x,
    ALIGN_RIGHT => term_w
      .saturating_sub(content_width)
      .saturating_sub(offset_x),
    _ => offset_x,
  }
}

/// Resolve a symbolic vertical position within the available terminal rows.
///
/// # Arguments
///
/// * `size` - The size.
/// * `y_anchor` - The y anchor.
/// * `content_height` - The content height in terminal rows.
/// * `offset_y` - The offset y.
pub fn resolve_y(size: Size, y_anchor: &str, content_height: u16, offset_y: u16) -> u16 {
  let term_h = size.height;
  match y_anchor {
    ALIGN_TOP => offset_y,
    ALIGN_MIDDLE => term_h.saturating_sub(content_height) / 2 + offset_y,
    ALIGN_BOTTOM => term_h
      .saturating_sub(content_height)
      .saturating_sub(offset_y),
    _ => offset_y,
  }
}

/// Resolve a rectangular request against its coordinate space and available bounds.
///
/// # Arguments
///
/// * `size` - The size.
/// * `x_anchor` - The x anchor.
/// * `y_anchor` - The y anchor.
/// * `content_width` - The content width in terminal columns.
/// * `content_height` - The content height in terminal rows.
/// * `offset_x` - The offset x.
/// * `offset_y` - The offset y.
pub fn resolve_rect(
  size: Size,
  x_anchor: &str,
  y_anchor: &str,
  content_width: u16,
  content_height: u16,
  offset_x: u16,
  offset_y: u16,
) -> Position {
  Position {
    x: resolve_x(size, x_anchor, content_width, offset_x),
    y: resolve_y(size, y_anchor, content_height, offset_y),
  }
}
