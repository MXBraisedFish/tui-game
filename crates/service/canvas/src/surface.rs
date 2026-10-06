//! Prepared drawing surfaces and their identities in a composed frame.

use tg_core_geometry::{Rect, Size};
use tg_core_style::{TextColor, TextStyle};

/// The identity of slice within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SliceId(
  /// The wrapped u64 value.
  pub u64,
);

/// The identity of scroll box within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScrollBoxId(
  /// The wrapped u64 value.
  pub u64,
);

/// The identity of a slice or scroll-box surface in a composed frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SurfaceId {
  /// The slice setting for surface id.
  Slice(SliceId),
  /// The scroll box setting for surface id.
  ScrollBox(ScrollBoxId),
}

/// The side of a viewport on which a scrollbar is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarSide {
  /// The right setting for scrollbar side.
  Right,
}

/// Characters and colors used to draw scrollbar tracks and thumbs.
///
/// # Fields
///
/// * `track_char` - The track char.
/// * `thumb_char` - The thumb char.
/// * `track_style` - The track style.
/// * `thumb_style` - The thumb style.
/// * `h_track_char` - The h track char.
/// * `h_thumb_char` - The h thumb char.
/// * `h_track_style` - The h track style.
/// * `h_thumb_style` - The h thumb style.
/// * `minimum_thumb_height` - The minimum thumb height in terminal rows.
/// * `side` - The side.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollbarStyle {
  /// The track char.
  pub track_char: char,

  /// The thumb char.
  pub thumb_char: char,

  /// The track style.
  pub track_style: TextStyle,

  /// The thumb style.
  pub thumb_style: TextStyle,

  /// The h track char.
  pub h_track_char: char,

  /// The h thumb char.
  pub h_thumb_char: char,

  /// The h track style.
  pub h_track_style: TextStyle,

  /// The h thumb style.
  pub h_thumb_style: TextStyle,

  /// The minimum thumb height in terminal rows.
  pub minimum_thumb_height: u16,

  /// The side.
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

/// Content bounds and scrollbar occupancy resolved against the visible viewport.
///
/// # Fields
///
/// * `viewport_rect` - The viewport rect.
/// * `content_viewport_rect` - The content viewport rect.
/// * `occupied_rect` - The occupied rect.
/// * `vertical_track_rect` - The vertical track rect.
/// * `horizontal_track_rect` - The horizontal track rect.
/// * `vertical_thumb_rect` - The vertical thumb rect.
/// * `horizontal_thumb_rect` - The horizontal thumb rect.
/// * `max_scroll_x` - The max scroll x.
/// * `max_scroll_y` - The max scroll y.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedScrollBoxLayout {
  /// The viewport rect.
  pub viewport_rect: Rect,

  /// The content viewport rect.
  pub content_viewport_rect: Rect,

  /// The occupied rect.
  pub occupied_rect: Rect,
  /// The vertical track rect.
  pub vertical_track_rect: Option<Rect>,
  /// The horizontal track rect.
  pub horizontal_track_rect: Option<Rect>,
  /// The vertical thumb rect.
  pub vertical_thumb_rect: Option<Rect>,
  /// The horizontal thumb rect.
  pub horizontal_thumb_rect: Option<Rect>,
  /// The max scroll x.
  pub max_scroll_x: u16,
  /// The max scroll y.
  pub max_scroll_y: u16,
}

/// The slice geometry and drawing data submitted for one composition pass.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `rect` - The rectangular region in terminal cells.
/// * `source_x` - The source x.
/// * `source_y` - The source y.
/// * `visible` - Whether this surface participates in composition.
/// * `opaque` - Whether empty cells cover lower surfaces.
/// * `background` - The background color override, or `None` to inherit the default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SliceFrame {
  /// The identifier of the owned object.
  pub id: SliceId,
  /// The rectangular region in terminal cells.
  pub rect: Rect,

  /// The source x.
  pub source_x: u16,
  /// The source y.
  pub source_y: u16,
  /// Whether this surface participates in composition.
  pub visible: bool,
  /// Whether empty cells cover lower surfaces.
  pub opaque: bool,
  /// The background color override, or `None` to inherit the default.
  pub background: Option<TextColor>,
}

/// The scroll box geometry and drawing data submitted for one composition pass.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `layout` - The service resolving terminal sizes and positions.
/// * `content_size` - The content size.
/// * `scroll_x` - The content offset from the viewport origin in terminal columns.
/// * `scroll_y` - The content offset from the viewport origin in terminal rows.
/// * `visible` - Whether this surface participates in composition.
/// * `opaque` - Whether empty cells cover lower surfaces.
/// * `scrollbar_style` - The scrollbar style.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollBoxFrame {
  /// The identifier of the owned object.
  pub id: ScrollBoxId,
  /// The service resolving terminal sizes and positions.
  pub layout: ResolvedScrollBoxLayout,
  /// The content size.
  pub content_size: Size,
  /// The content offset from the viewport origin in terminal columns.
  pub scroll_x: u16,
  /// The content offset from the viewport origin in terminal rows.
  pub scroll_y: u16,
  /// Whether this surface participates in composition.
  pub visible: bool,
  /// Whether empty cells cover lower surfaces.
  pub opaque: bool,
  /// The scrollbar style.
  pub scrollbar_style: ScrollbarStyle,
}

/// The surface geometry and drawing data submitted for one composition pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceFrame {
  /// The slice setting for surface frame.
  Slice(SliceFrame),
  /// The scroll box setting for surface frame.
  ScrollBox(ScrollBoxFrame),
}

impl SurfaceFrame {
  /// Return the current id.
  pub fn id(&self) -> SurfaceId {
    match self {
      Self::Slice(frame) => SurfaceId::Slice(frame.id),
      Self::ScrollBox(frame) => SurfaceId::ScrollBox(frame.id),
    }
  }
}
