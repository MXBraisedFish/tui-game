use std::collections::HashMap;

use crate::host_engine::services::Rect;

use super::types::{HyperlinkId, HyperlinkOptions};

#[derive(Clone, Copy)]
pub(crate) struct HyperlinkHit {
  pub rect: Rect,
  pub order: u64,
  pub surface_rank: usize,
}

pub(crate) struct HyperlinkState {
  pub options: HyperlinkOptions,
  pub hit: Option<HyperlinkHit>,
}

pub(crate) struct HyperlinkObjects {
  pub next_id: u64,
  pub links: HashMap<HyperlinkId, HyperlinkState>,
  pub pressed: Option<HyperlinkId>,
}

impl HyperlinkObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      links: HashMap::new(),
      pressed: None,
    }
  }

  pub(crate) fn clear_hits(&mut self) {
    self.pressed = self
      .pressed
      .filter(|id| self.links.get(id).is_some_and(|state| state.hit.is_some()));
    for state in self.links.values_mut() {
      state.hit = None;
    }
  }

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
