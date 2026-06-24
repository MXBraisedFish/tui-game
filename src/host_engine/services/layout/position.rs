use super::types::{Position, Size};

// ── 水平对齐常量 ──

pub const ALIGN_LEFT: &str = "left";
pub const ALIGN_CENTER: &str = "center";
pub const ALIGN_RIGHT: &str = "right";

// ── 垂直对齐常量 ──

pub const ALIGN_TOP: &str = "top";
pub const ALIGN_MIDDLE: &str = "middle";
pub const ALIGN_BOTTOM: &str = "bottom";

// ── 定位计算 ──

/// 根据水平锚点和内容宽度，计算 x 起始坐标（均相对于终端画布）。
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

/// 根据垂直锚点和内容高度，计算 y 起始坐标（均相对于终端画布）。
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

/// 同时计算水平和垂直起始坐标，返回 Position。
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
