use super::types::{Position, Size};

/// Horizontal alignment: left.
pub const ALIGN_LEFT: &str = "left";

/// Horizontal alignment: center.
pub const ALIGN_CENTER: &str = "center";

/// Horizontal alignment: right.
pub const ALIGN_RIGHT: &str = "right";

/// Vertical alignment: top.
pub const ALIGN_TOP: &str = "top";

/// Vertical alignment: middle.
pub const ALIGN_MIDDLE: &str = "middle";

/// Vertical alignment: bottom.
pub const ALIGN_BOTTOM: &str = "bottom";

/// Returns the X coordinate for the horizontal anchor and content width.
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

/// Returns the Y coordinate for the vertical anchor and content height.
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

/// Returns the position for the anchors and content size.
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
