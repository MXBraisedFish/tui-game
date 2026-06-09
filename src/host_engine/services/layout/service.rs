use super::{measure, position};

/// 布局服务：提供文本测量和居中计算的宿主侧 API
/// 本身无状态，所有方法委托给 measure / position 纯函数
pub struct LayoutService;

impl LayoutService {
  pub fn new() -> Self {
    Self
  }

  // ── 测量 ──

  pub fn measure_width(&self, text: &str) -> u16 {
    measure::measure_width(text)
  }

  pub fn measure_height(&self, text: &str) -> u16 {
    measure::measure_height(text)
  }

  pub fn measure_size(&self, text: &str) -> (u16, u16) {
    measure::measure_size(text)
  }

  // ── 居中 ──

  pub fn center_x(&self, area_width: u16, content_width: u16) -> u16 {
    position::center_x(area_width, content_width)
  }

  pub fn center_y(&self, area_height: u16, content_height: u16) -> u16 {
    position::center_y(area_height, content_height)
  }

  pub fn center_pos(
    &self,
    area_width: u16,
    area_height: u16,
    content_width: u16,
    content_height: u16,
  ) -> (u16, u16) {
    position::center_pos(area_width, area_height, content_width, content_height)
  }
}
