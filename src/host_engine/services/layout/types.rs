/// 尺寸（宽 x 高）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
  pub width: u16,
  pub height: u16,
}

/// 二维坐标位置
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Position {
  pub x: u16,
  pub y: u16,
}

/// 矩形区域
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
  pub x: u16,
  pub y: u16,
  pub width: u16,
  pub height: u16,
}

impl Rect {
  /// 判断点是否在矩形内部（不含右边界和下边界）
  pub fn contains(&self, px: u16, py: u16) -> bool {
    px >= self.x
      && px < self.x.saturating_add(self.width)
      && py >= self.y
      && py < self.y.saturating_add(self.height)
  }
}
