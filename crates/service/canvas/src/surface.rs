//! Identifiers, scrollbar styles and per-frame descriptions of drawing surfaces (slices and scroll
//! boxes).
//!
//! The owner of the UI objects (widget) builds a list of [`SurfaceFrame`]s every frame and hands it
//! to the canvas; the canvas never reads the UI object pool.

use tg_core_geometry::{Rect, Size};
use tg_core_style::{TextColor, TextStyle};

/// The unique identifier of a slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SliceId(pub u64);

/// The unique identifier of a scrollable drawing surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScrollBoxId(pub u64);

/// The unified identifier of a stackable developer drawing surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SurfaceId {
  Slice(SliceId),
  ScrollBox(ScrollBoxId),
}

/// The side a scrollbar is placed on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarSide {
  Right,
}

/// The style of a scrollbar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollbarStyle {
  /// Track character of the vertical scrollbar.
  pub track_char: char,
  /// Thumb character of the vertical scrollbar.
  pub thumb_char: char,
  /// Track style of the vertical scrollbar.
  pub track_style: TextStyle,
  /// Thumb style of the vertical scrollbar.
  pub thumb_style: TextStyle,
  /// Track character of the horizontal scrollbar.
  pub h_track_char: char,
  /// Thumb character of the horizontal scrollbar.
  pub h_thumb_char: char,
  /// Track style of the horizontal scrollbar.
  pub h_track_style: TextStyle,
  /// Thumb style of the horizontal scrollbar.
  pub h_thumb_style: TextStyle,
  /// Minimum thumb height/width (default 1).
  pub minimum_thumb_height: u16,
  /// Side the scrollbar is placed on.
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

/// The fully resolved area layout of a scroll box.
///
/// All scrolling, clipping, drawing and hit testing must use this result; never derive the
/// scrollbar visibility from the options again, so that different stages cannot disagree about
/// which area a cell belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedScrollBoxLayout {
  /// The component viewport: `options.rect` clipped to the developer viewport.
  pub viewport_rect: Rect,
  /// The visible content area, excluding the cells actually occupied by scrollbars.
  pub content_viewport_rect: Rect,
  /// The area occupied by the viewport together with the external scrollbars.
  pub occupied_rect: Rect,
  pub vertical_track_rect: Option<Rect>,
  pub horizontal_track_rect: Option<Rect>,
  pub vertical_thumb_rect: Option<Rect>,
  pub horizontal_thumb_rect: Option<Rect>,
  pub max_scroll_x: u16,
  pub max_scroll_y: u16,
}

/// The description of a slice in the current frame: its resolved position and display attributes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SliceFrame {
  pub id: SliceId,
  pub rect: Rect,
  /// Source position inside the configured slice that maps to `rect`'s top-left cell.
  pub source_x: u16,
  pub source_y: u16,
  pub visible: bool,
  pub opaque: bool,
  pub background: Option<TextColor>,
}

/// The description of a scroll box in the current frame: its resolved area layout, content size
/// and scroll position.
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

/// The description of one drawing surface in the current frame, passed to
/// [`CanvasService::prepare`](super::CanvasService::prepare) in stacking order.
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
