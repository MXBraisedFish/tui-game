use crate::host_engine::services::{Rect, TextColor, TextStyle};

/// 可滚动绘制面唯一标识。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScrollBoxId(pub u64);

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

/// 滚动条放置侧。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarSide {
  Right,
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

/// 滚动条样式。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollbarStyle {
  /// 垂直滚动条轨道字符。
  pub track_char: char,
  /// 垂直滚动条滑块字符。
  pub thumb_char: char,
  /// 垂直滚动条轨道样式。
  pub track_style: TextStyle,
  /// 垂直滚动条滑块样式。
  pub thumb_style: TextStyle,
  /// 水平滚动条轨道字符。
  pub h_track_char: char,
  /// 水平滚动条滑块字符。
  pub h_thumb_char: char,
  /// 水平滚动条轨道样式。
  pub h_track_style: TextStyle,
  /// 水平滚动条滑块样式。
  pub h_thumb_style: TextStyle,
  /// 滑块最小高度/宽度（默认 1）。
  pub minimum_thumb_height: u16,
  /// 滚动条放置侧。
  pub side: ScrollbarSide,
}

impl Default for ScrollbarStyle {
  fn default() -> Self {
    Self {
      track_char: '│',
      thumb_char: '█',
      track_style: TextStyle {
        foreground: Some(TextColor::Rgb {
          r: 85,
          g: 87,
          b: 83,
        }),
        ..Default::default()
      },
      thumb_style: TextStyle {
        foreground: Some(TextColor::Rgb {
          r: 220,
          g: 223,
          b: 218,
        }),
        ..Default::default()
      },
      h_track_char: '─',
      h_thumb_char: '█',
      h_track_style: TextStyle {
        foreground: Some(TextColor::Rgb {
          r: 85,
          g: 87,
          b: 83,
        }),
        ..Default::default()
      },
      h_thumb_style: TextStyle {
        foreground: Some(TextColor::Rgb {
          r: 220,
          g: 223,
          b: 218,
        }),
        ..Default::default()
      },
      minimum_thumb_height: 1,
      side: ScrollbarSide::Right,
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
