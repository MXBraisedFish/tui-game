//! Owned component state and per-frame hit-test caches.

use std::collections::{HashMap, VecDeque};

use tg_core_input::MouseButton;

use super::types::{ScrollBoxEvent, ScrollBoxId, ScrollBoxOptions, ScrollbarAxis};

/// The retained state of scroll box drag.
///
/// # Fields
///
/// * `scroll_box_id` - The identifier of the scroll box.
/// * `axis` - The axis.
/// * `button` - The mouse button to query.
/// * `drag_start_mouse` - The drag start mouse.
/// * `drag_start_thumb_pos` - The drag start thumb pos.
/// * `thumb_size` - The thumb size.
/// * `track_size` - The track size.
/// * `max_scroll` - The max scroll.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ScrollBoxDragState {
  /// The identifier of the scroll box.
  pub scroll_box_id: ScrollBoxId,
  /// The axis.
  pub axis: ScrollbarAxis,
  /// The mouse button to query.
  pub button: MouseButton,

  /// The drag start mouse.
  pub drag_start_mouse: u16,

  /// The drag start thumb pos.
  pub drag_start_thumb_pos: u16,

  /// The thumb size.
  pub thumb_size: u16,

  /// The track size.
  pub track_size: u16,

  /// The max scroll.
  pub max_scroll: u16,
}

impl ScrollBoxDragState {
  /// Resolve an anchored scrollbar drag into a clamped content scroll position.
  pub(crate) fn scroll_from_mouse(&self, mouse_pos: u16) -> u16 {
    let travel = self.track_size.saturating_sub(self.thumb_size);
    if travel == 0 {
      return 0;
    }
    // Calculate drag displacement from the initial pointer and thumb positions, not the previous
    // frame.

    let thumb_pos = (mouse_pos as i32 - self.drag_start_mouse as i32
      + self.drag_start_thumb_pos as i32)
      .max(0)
      .min(travel as i32) as u16;
    (thumb_pos as u32 * self.max_scroll as u32 / travel as u32) as u16
  }
}

/// The retained state of scroll box.
///
/// # Fields
///
/// * `options` - The scroll box options carried by this scroll box state.
/// * `scroll_x` - The content offset from the viewport origin in terminal columns.
/// * `scroll_y` - The content offset from the viewport origin in terminal rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollBoxState {
  /// The scroll box options carried by this scroll box state.
  pub options: ScrollBoxOptions,
  /// The content offset from the viewport origin in terminal columns.
  pub scroll_x: u16,
  /// The content offset from the viewport origin in terminal rows.
  pub scroll_y: u16,
}

/// The collection of owned scroll box instances and their queued events.
///
/// # Fields
///
/// * `next_id` - The identifier of the next.
/// * `boxes` - The boxes indexed by their declared keys.
/// * `events` - Pending events retained in delivery order.
/// * `drag` - The drag.
pub struct ScrollBoxObjects {
  /// The identifier of the next.
  pub next_id: u64,
  /// The boxes indexed by their declared keys.
  pub boxes: HashMap<ScrollBoxId, ScrollBoxState>,
  /// Pending events retained in delivery order.
  pub(crate) events: VecDeque<ScrollBoxEvent>,
  /// The drag.
  pub(crate) drag: Option<ScrollBoxDragState>,
}

impl ScrollBoxObjects {
  /// Create a scroll box objects with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      boxes: HashMap::new(),
      events: VecDeque::new(),
      drag: None,
    }
  }
}
