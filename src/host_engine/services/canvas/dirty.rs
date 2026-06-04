//! 脏区间追踪模块
//!
//! 用行+列区间 `DirtySpan` 替代仅行级的脏标记，
//! 使差异渲染能精确跳过同一行中未被修改的列区间，
//! 进一步减少终端 I/O 操作。

/// 脏区间
///
/// 表示画布上一段需要重绘的矩形子区域。
/// 同行的多个重叠或相邻区间可被合并以降低渲染开销。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirtySpan {
  pub y: u16,          // 行号
  pub start_x: u16,    // 该行中脏区间的起始列
  pub end_x: u16,      // 该行中脏区间的结束列（不包含）
}

impl DirtySpan {
  /// 创建一个新的脏区间
  pub fn new(y: u16, start_x: u16, end_x: u16) -> Self {
    Self { y, start_x, end_x }
  }

  /// 该区间是否为空（起始位置 >= 结束位置）
  pub fn is_empty(&self) -> bool {
    self.start_x >= self.end_x
  }

  /// 尝试将另一个脏区间合并到自身
  ///
  /// 仅当两个区间在同一行且有重叠或相邻时才能合并。
  /// 合并后自身范围将扩展以包含对方。
  pub fn merge_if_possible(&mut self, other: DirtySpan) -> bool {
    if self.y != other.y {
      return false;
    }
    if other.end_x < self.start_x {
      return false;
    }
    if other.start_x > self.end_x {
      return false;
    }
    self.start_x = self.start_x.min(other.start_x);
    self.end_x = self.end_x.max(other.end_x);
    true
  }
}
