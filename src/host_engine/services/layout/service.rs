use super::types::{Position, Size};
use super::{measure, position};
use crate::host_engine::services::RichTextParams;

/// 布局服务：提供文本测量和定位计算的宿主侧 API。
/// 本身无状态，所有方法委托给 measure / position 纯函数。
pub struct LayoutService;

impl LayoutService {
  pub fn new() -> Self {
    Self
  }

  // ── 测量 ──

  pub fn get_text_size(
    &self,
    text: &str,
    params: Option<&RichTextParams>,
  ) -> Size {
    measure::get_text_size(text, params)
  }

  pub fn get_text_width(
    &self,
    text: &str,
    params: Option<&RichTextParams>,
  ) -> u16 {
    measure::get_text_width(text, params)
  }

  pub fn get_text_height(
    &self,
    text: &str,
    params: Option<&RichTextParams>,
  ) -> u16 {
    measure::get_text_height(text, params)
  }

  pub fn get_terminal_size(&self) -> Size {
    measure::get_terminal_size()
  }

  // ── 定位 ──

  pub fn resolve_x(
    &self,
    x_anchor: &str,
    content_width: u16,
    offset_x: u16,
  ) -> u16 {
    position::resolve_x(x_anchor, content_width, offset_x)
  }

  pub fn resolve_y(
    &self,
    y_anchor: &str,
    content_height: u16,
    offset_y: u16,
  ) -> u16 {
    position::resolve_y(y_anchor, content_height, offset_y)
  }

  pub fn resolve_rect(
    &self,
    x_anchor: &str,
    y_anchor: &str,
    content_width: u16,
    content_height: u16,
    offset_x: u16,
    offset_y: u16,
  ) -> Position {
    position::resolve_rect(
      x_anchor, y_anchor, content_width, content_height, offset_x, offset_y,
    )
  }

  // ── 对齐常量（透传） ──

  pub const ALIGN_LEFT: &'static str = position::ALIGN_LEFT;
  pub const ALIGN_CENTER: &'static str = position::ALIGN_CENTER;
  pub const ALIGN_RIGHT: &'static str = position::ALIGN_RIGHT;
  pub const ALIGN_TOP: &'static str = position::ALIGN_TOP;
  pub const ALIGN_MIDDLE: &'static str = position::ALIGN_MIDDLE;
  pub const ALIGN_BOTTOM: &'static str = position::ALIGN_BOTTOM;
}
