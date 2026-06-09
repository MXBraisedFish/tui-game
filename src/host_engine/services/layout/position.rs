/// 水平居中：给定区域宽度和内容宽度，返回 x 坐标
pub fn center_x(area_width: u16, content_width: u16) -> u16 {
  area_width.saturating_sub(content_width) / 2
}

/// 垂直居中：给定区域高度和内容高度，返回 y 坐标
pub fn center_y(area_height: u16, content_height: u16) -> u16 {
  area_height.saturating_sub(content_height) / 2
}

/// 居中位置：给定区域宽高和内容宽高，返回 (x, y)
pub fn center_pos(
  area_width: u16,
  area_height: u16,
  content_width: u16,
  content_height: u16,
) -> (u16, u16) {
  (
    center_x(area_width, content_width),
    center_y(area_height, content_height),
  )
}
