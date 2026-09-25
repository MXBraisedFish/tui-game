//! 绘制面（切片与滚动盒子）的标识、滚动条样式与每帧描述。
//!
//! 持有 UI 对象的一方（widget）每帧生成 [`SurfaceFrame`] 列表交给画布，画布不读取 UI 对象池。

use crate::host_engine::services::{Rect, Size, TextColor, TextStyle};

/// 切片唯一标识
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SliceId(pub u64);

/// 可滚动绘制面唯一标识。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScrollBoxId(pub u64);

/// 开发者可叠放绘制面的统一标识。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SurfaceId {
  Slice(SliceId),
  ScrollBox(ScrollBoxId),
}

/// 滚动条放置侧。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarSide {
  Right,
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

/// ScrollBox 的完整区域解析结果。
///
/// 所有滚动、裁剪、绘制与命中检测都必须使用该结果，禁止再次根据 options
/// 推导滚动条可见性，避免不同阶段对同一格子的归属产生分歧。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedScrollBoxLayout {
  /// `options.rect` 经 Developer Viewport 裁剪后的组件 viewport。
  pub viewport_rect: Rect,
  /// 排除滚动条实际占用格子后的内容可视区域。
  pub content_viewport_rect: Rect,
  /// viewport 与外置滚动条共同占用的区域。
  pub occupied_rect: Rect,
  pub vertical_track_rect: Option<Rect>,
  pub horizontal_track_rect: Option<Rect>,
  pub vertical_thumb_rect: Option<Rect>,
  pub horizontal_thumb_rect: Option<Rect>,
  pub max_scroll_x: u16,
  pub max_scroll_y: u16,
}

/// 切片在本帧的描述：已解析的位置与显示属性。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SliceFrame {
  pub id: SliceId,
  pub rect: Rect,
  pub visible: bool,
  pub opaque: bool,
  pub background: Option<TextColor>,
}

/// 滚动盒子在本帧的描述：已解析的区域布局、内容尺寸与滚动位置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollBoxFrame {
  pub id: ScrollBoxId,
  pub layout: ResolvedScrollBoxLayout,
  pub content_size: Size,
  pub scroll_x: u16,
  pub scroll_y: u16,
  pub visible: bool,
  pub opaque: bool,
  pub scrollbar_style: ScrollbarStyle,
}

/// 一个绘制面在本帧的描述，按叠放顺序传给 [`CanvasService::prepare`](super::CanvasService::prepare)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceFrame {
  Slice(SliceFrame),
  ScrollBox(ScrollBoxFrame),
}

impl SurfaceFrame {
  pub fn id(&self) -> SurfaceId {
    match self {
      Self::Slice(frame) => SurfaceId::Slice(frame.id),
      Self::ScrollBox(frame) => SurfaceId::ScrollBox(frame.id),
    }
  }
}
