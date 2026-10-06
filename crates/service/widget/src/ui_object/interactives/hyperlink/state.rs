//! Owned component state and per-frame hit-test caches.

use std::collections::HashMap;

use tg_service_layout::Rect;

use super::types::{HyperlinkId, HyperlinkOptions};

/// The hyperlink hit representation used by this module.
///
/// # Fields
///
/// * `rect` - The rectangular region in terminal cells.
/// * `order` - The order.
/// * `surface_rank` - The surface rank.
#[derive(Clone, Copy)]
pub(crate) struct HyperlinkHit {
  /// The rectangular region in terminal cells.
  pub rect: Rect,
  /// The order.
  pub order: u64,
  /// The surface rank.
  pub surface_rank: usize,
}

/// The retained state of hyperlink.
///
/// # Fields
///
/// * `options` - The hyperlink options carried by this hyperlink state.
/// * `hit` - The hit.
pub(crate) struct HyperlinkState {
  /// The hyperlink options carried by this hyperlink state.
  pub options: HyperlinkOptions,
  /// The hit.
  pub hit: Option<HyperlinkHit>,
}

/// The collection of owned hyperlink instances and their queued events.
///
/// # Fields
///
/// * `next_id` - The identifier of the next.
/// * `links` - The links indexed by their declared keys.
/// * `pressed` - The pressed.
pub(crate) struct HyperlinkObjects {
  /// The identifier of the next.
  pub next_id: u64,
  /// The links indexed by their declared keys.
  pub links: HashMap<HyperlinkId, HyperlinkState>,
  /// The pressed.
  pub pressed: Option<HyperlinkId>,
}

impl HyperlinkObjects {
  /// Create a hyperlink objects with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      links: HashMap::new(),
      pressed: None,
    }
  }

  /// Discard hit rectangles from the previous drawing pass.
  pub(crate) fn clear_hits(&mut self) {
    self.pressed = self
      .pressed
      .filter(|id| self.links.get(id).is_some_and(|state| state.hit.is_some()));
    for state in self.links.values_mut() {
      state.hit = None;
    }
  }

  /// Find the component's current hit rectangle containing the pointer position.
  pub(crate) fn hit(&self, x: u16, y: u16) -> Option<(HyperlinkId, (usize, u64))> {
    self
      .links
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
}
