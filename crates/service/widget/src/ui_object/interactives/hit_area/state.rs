//! Owned component state and per-frame hit-test caches.

use std::collections::HashMap;

use tg_core_input::MouseButton;
use tg_service_layout::Rect;

use super::types::{HitAreaId, HitAreaOptions};

/// A retained copy of hit data for later inspection or replay.
///
/// # Fields
///
/// * `rect` - The rectangular region in terminal cells.
/// * `order` - The order.
/// * `origin` - The origin.
/// * `surface_rank` - The surface rank.
#[derive(Clone, Copy)]
pub(crate) struct HitSnapshot {
  /// The rectangular region in terminal cells.
  pub rect: Rect,
  /// The order.
  pub order: u64,
  /// The origin.
  pub origin: (i32, i32),
  /// The surface rank.
  pub surface_rank: usize,
}

/// The retained state of press.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `last_x` - The last x.
/// * `last_y` - The last y.
#[derive(Clone, Copy)]
pub(crate) struct PressState {
  /// The identifier of the owned object.
  pub id: HitAreaId,
  /// The last x.
  pub last_x: u16,
  /// The last y.
  pub last_y: u16,
}

/// The retained state of hit area.
///
/// # Fields
///
/// * `hit` - The hit.
/// * `options` - The hit area options carried by this hit area state.
pub(crate) struct HitAreaState {
  /// The hit.
  pub hit: Option<HitSnapshot>,
  /// The hit area options carried by this hit area state.
  pub options: HitAreaOptions,
}

/// The collection of owned hit area instances and their queued events.
///
/// # Fields
///
/// * `next_id` - The identifier of the next.
/// * `areas` - The areas indexed by their declared keys.
/// * `hovered` - The hovered.
/// * `pressed` - The pressed indexed by their declared keys.
/// * `pointer` - The pointer.
/// * `physical_pointer` - The physical pointer.
pub(crate) struct HitAreaObjects {
  /// The identifier of the next.
  pub next_id: u64,
  /// The areas indexed by their declared keys.
  pub areas: HashMap<HitAreaId, HitAreaState>,
  /// The hovered.
  pub hovered: Option<HitAreaId>,
  /// The pressed indexed by their declared keys.
  pub pressed: HashMap<MouseButton, PressState>,
  /// The pointer.
  pub pointer: Option<(u16, u16)>,
  /// The physical pointer.
  pub physical_pointer: Option<(u16, u16)>,
}

impl HitAreaObjects {
  /// Create a hit area objects with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      areas: HashMap::new(),
      hovered: None,
      pressed: HashMap::new(),
      pointer: None,
      physical_pointer: None,
    }
  }

  /// Find the component's current hit rectangle containing the pointer position.
  pub(crate) fn hit(&self, x: u16, y: u16) -> Option<(HitAreaId, (usize, u64))> {
    self
      .areas
      .iter()
      .filter_map(|(id, state)| {
        let hit = state.hit?;
        hit
          .rect
          .contains(x, y)
          .then_some((*id, (hit.surface_rank, hit.order)))
      })
      .max_by_key(|(_, order)| *order)
  }

  /// Discard hit rectangles from the previous drawing pass.
  pub(crate) fn clear_hits(&mut self) {
    for state in self.areas.values_mut() {
      state.hit = None;
    }
  }
}
