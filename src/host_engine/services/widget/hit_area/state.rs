use std::collections::HashMap;

use crate::host_engine::services::{MouseButton, Rect};

use super::types::{HitAreaId, HitAreaOptions};

#[derive(Clone, Copy)]
pub(crate) struct HitSnapshot {
  pub rect: Rect,
  pub order: u64,
  pub origin: (u16, u16),
  pub surface_rank: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct PressState {
  pub id: HitAreaId,
  pub last_x: u16,
  pub last_y: u16,
}

pub(crate) struct HitAreaState {
  pub hit: Option<HitSnapshot>,
  pub options: HitAreaOptions,
}

pub(crate) struct HitAreaObjects {
  pub next_id: u64,
  pub areas: HashMap<HitAreaId, HitAreaState>,
  pub hovered: Option<HitAreaId>,
  pub pressed: HashMap<MouseButton, PressState>,
  pub pointer: Option<(u16, u16)>,
  pub physical_pointer: Option<(u16, u16)>,
}

impl HitAreaObjects {
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

  pub(crate) fn clear_hits(&mut self) {
    for state in self.areas.values_mut() {
      state.hit = None;
    }
  }
}
