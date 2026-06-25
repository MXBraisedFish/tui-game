use super::scroll_box::ScrollBoxId;
use super::slice::SliceId;

/// 开发者可叠放绘制面的统一标识。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SurfaceId {
  Slice(SliceId),
  ScrollBox(ScrollBoxId),
}
