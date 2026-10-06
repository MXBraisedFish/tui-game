//! Identifiers, configuration, states, and events shared by this module.

use tg_service_layout::Rect;

pub use tg_service_canvas::{ScrollBoxId, ScrollbarSide, ScrollbarStyle};

/// The allowed scrolling directions for content outside its viewport.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
  /// The hidden setting for overflow.
  Hidden,
  /// The auto setting for overflow.
  Auto,
}

/// The policy selecting when a scrollbar should be displayed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarVisibility {
  /// The auto setting for scrollbar visibility.
  Auto,
  /// The always setting for scrollbar visibility.
  Always,
  /// The never setting for scrollbar visibility.
  Never,
}

/// Overlay, inside, or reserved-space scrollbar placement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarLayout {
  /// The overlay setting for scrollbar layout.
  Overlay,

  /// The reserve space setting for scrollbar layout.
  ReserveSpace,

  /// The inside setting for scrollbar layout.
  #[default]
  Inside,
}

/// The horizontal or vertical dimension controlled by a scrollbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScrollbarAxis {
  /// The vertical setting for scrollbar axis.
  Vertical,
  /// The horizontal setting for scrollbar axis.
  Horizontal,
}

/// The visibility and layout settings for one scroll axis.
///
/// # Fields
///
/// * `vertical` - The vertical.
/// * `horizontal` - The horizontal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollbarPolicy {
  /// The vertical.
  pub vertical: ScrollbarVisibility,
  /// The horizontal.
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

/// Configuration values controlling scroll box behavior.
///
/// # Fields
///
/// * `rect` - The rectangular region in terminal cells.
/// * `content_width` - The content width in terminal columns.
/// * `content_height` - The content height in terminal rows.
/// * `overflow_y` - The overflow y.
/// * `overflow_x` - The overflow x.
/// * `scrollbar` - The scrollbar.
/// * `scrollbar_style` - The scrollbar style.
/// * `scrollbar_layout` - The scrollbar layout.
/// * `visible` - Whether this surface participates in composition.
/// * `opaque` - Whether empty cells cover lower surfaces.
/// * `mouse_wheel` - The mouse wheel.
/// * `wheel_step` - The wheel step.
/// * `h_wheel_step` - The h wheel step.
/// * `emit_scroll_events` - The emit scroll events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollBoxOptions {
  /// The rectangular region in terminal cells.
  pub rect: Rect,
  /// The content width in terminal columns.
  pub content_width: u16,
  /// The content height in terminal rows.
  pub content_height: u16,
  /// The overflow y.
  pub overflow_y: Overflow,
  /// The overflow x.
  pub overflow_x: Overflow,
  /// The scrollbar.
  pub scrollbar: ScrollbarPolicy,
  /// The scrollbar style.
  pub scrollbar_style: ScrollbarStyle,
  /// The scrollbar layout.
  pub scrollbar_layout: ScrollbarLayout,
  /// Whether this surface participates in composition.
  pub visible: bool,
  /// Whether empty cells cover lower surfaces.
  pub opaque: bool,
  /// The mouse wheel.
  pub mouse_wheel: bool,

  /// The wheel step.
  pub wheel_step: u16,

  /// The h wheel step.
  pub h_wheel_step: u16,
  /// The emit scroll events.
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

/// A scroll box event payload queued for its owning consumer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollBoxEvent {
  /// A scrolled notification delivered to the owning consumer.
  Scrolled {
    /// The identifier of the owned object.
    id: ScrollBoxId,
    /// The horizontal coordinate in terminal cells.
    x: u16,
    /// The vertical coordinate in terminal cells.
    y: u16,
  },
}
