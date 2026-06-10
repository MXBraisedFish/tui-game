/// 尺寸（宽度 + 高度）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
  pub width: u16,
  pub height: u16,
}

/// 坐标位置（x + y）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Position {
  pub x: u16,
  pub y: u16,
}

/// 矩形区域（用于命中测试）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
  pub x: u16,
  pub y: u16,
  pub width: u16,
  pub height: u16,
}

impl Rect {
  /// 判断点 `(px, py)` 是否在矩形区域内。
  pub fn contains(&self, px: u16, py: u16) -> bool {
    px >= self.x
      && px < self.x.saturating_add(self.width)
      && py >= self.y
      && py < self.y.saturating_add(self.height)
  }
}
