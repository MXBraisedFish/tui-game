use crate::host_engine::services::Rect;

pub use crate::host_engine::services::canvas::{ScrollBoxId, ScrollbarSide, ScrollbarStyle};

/// 溢出处理方式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
  Hidden,
  Auto,
}

/// 滚动条显示策略。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarVisibility {
  Auto,
  Always,
  Never,
}

/// 滚动条占位策略。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarLayout {
  /// 滚动条绘制在 viewport 内；其占据的格子不计入内容可视区域。
  Overlay,
  /// 滚动条占用一列/行，绘制在 viewport 外部，内容可视区域减少 1。
  ReserveSpace,
  /// 滚动条绘制在 viewport 内部最右侧/最底部，内容可视区域减少 1（不被遮挡）。
  Inside,
}

impl Default for ScrollbarLayout {
  fn default() -> Self {
    Self::Inside
  }
}

/// 滚动条轴向（内部使用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScrollbarAxis {
  Vertical,
  Horizontal,
}

/// 滚动条策略。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollbarPolicy {
  pub vertical: ScrollbarVisibility,
  pub horizontal: ScrollbarVisibility,
}

impl Default for ScrollbarPolicy {
  fn default() -> Self {
    Self {
      vertical: ScrollbarVisibility::Auto,
      horizontal: ScrollbarVisibility::Never,
    }
  }
}

/// 可滚动绘制面配置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollBoxOptions {
  pub rect: Rect,
  pub content_width: u16,
  pub content_height: u16,
  pub overflow_y: Overflow,
  pub overflow_x: Overflow,
  pub scrollbar: ScrollbarPolicy,
  pub scrollbar_style: ScrollbarStyle,
  pub scrollbar_layout: ScrollbarLayout,
  pub visible: bool,
  pub opaque: bool,
  pub mouse_wheel: bool,
  /// 纵向滚轮步长（每次滚轮滚动的行数）。
  pub wheel_step: u16,
  /// 横向滚轮步长（每次滚轮滚动的列数）。
  pub h_wheel_step: u16,
  pub emit_scroll_events: bool,
}

impl Default for ScrollBoxOptions {
  fn default() -> Self {
    Self {
      rect: Rect::default(),
      content_width: 0,
      content_height: 0,
      overflow_y: Overflow::Auto,
      overflow_x: Overflow::Hidden,
      scrollbar: ScrollbarPolicy::default(),
      scrollbar_style: ScrollbarStyle::default(),
      scrollbar_layout: ScrollbarLayout::default(),
      visible: true,
      opaque: true,
      mouse_wheel: true,
      wheel_step: 3,
      h_wheel_step: 2,
      emit_scroll_events: false,
    }
  }
}

/// 滚动盒子事件。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollBoxEvent {
  Scrolled { id: ScrollBoxId, x: u16, y: u16 },
}
