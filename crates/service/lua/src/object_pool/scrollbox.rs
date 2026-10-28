//! Session-owned scrollbox components and declared surface parent relationships.

use super::*;
use crate::{LuaDrawTarget, LuaScrollBoxEvent};
use tg_service_widget::{ScrollBoxWidget, SliceId};

/// A script scrollbox paired with its private content surface and original color expression.
pub(crate) struct LuaScrollboxBinding {
  pub widget: ScrollBoxWidget,
  pub surface: SliceId,
  pub background: String,
}
#[derive(Clone, Copy)]
/// A frame-local parent, position and submission identity for an embedded surface.
pub(crate) struct SurfacePlacement {
  pub parent: LuaDrawTarget,
  pub position: (i32, i32),
  pub sequence: u64,
}
impl LuaObjectPool {
  pub(crate) fn place_surface(
    &mut self,
    id: SliceId,
    parent: LuaDrawTarget,
    position: (i32, i32),
  ) -> Result<u64, &'static str> {
    let old = self.surface_placements.get(&id).copied();
    self.surface_sequence += 1;
    self.surface_placements.insert(
      id,
      SurfacePlacement {
        parent,
        position,
        sequence: self.surface_sequence,
      },
    );
    let invalid = self.surface_placements.keys().find_map(|start| {
      let mut current = *start;
      let mut visited = std::collections::HashSet::new();
      loop {
        if !visited.insert(current) {
          return Some("surface parent cycle");
        }
        if visited.len() > 32 {
          return Some("surface nesting exceeds 32 levels");
        }
        match self
          .surface_placements
          .get(&current)
          .map(|placement| placement.parent)
        {
          Some(LuaDrawTarget::Slice(parent)) => current = parent,
          _ => return None,
        }
      }
    });
    if let Some(error) = invalid {
      if let Some(previous) = old {
        self.surface_placements.insert(id, previous);
      } else {
        self.surface_placements.remove(&id);
      }
      return Err(error);
    }
    Ok(self.surface_sequence)
  }
  pub(crate) fn scrollbox_id_for_surface(&self, id: SliceId) -> Option<u64> {
    self
      .scrollboxes
      .iter()
      .find_map(|(key, binding)| (binding.surface == id).then_some(*key))
  }
  pub(crate) fn take_scrollbox_events(&mut self) -> Vec<LuaEventData> {
    self
      .scrollboxes
      .iter_mut()
      .filter_map(|(id, binding)| {
        binding
          .widget
          .take_scroll_event()
          .map(|(x, y)| LuaEventData::ScrollBox(LuaScrollBoxEvent { id: *id, x, y }))
      })
      .collect()
  }
  pub(crate) fn delete_scrollbox(&mut self, id: u64) -> bool {
    if let Some(binding) = self.scrollboxes.get(&id) {
      self.invalidate_surface(binding.surface);
    }
    let Some(binding) = self.scrollboxes.remove(&id) else {
      return false;
    };
    self.surface_placements.remove(&binding.surface);
    true
  }

  pub(crate) fn clear_scrollboxes(&mut self) {
    for binding in std::mem::take(&mut self.scrollboxes).into_values() {
      self.ui.invalidate_pointer_scope(binding.widget.scope());
      self.surface_placements.remove(&binding.surface);
    }
  }

  pub(crate) fn invalidate_surface(&mut self, surface: SliceId) {
    let mut descendants = std::collections::HashSet::from([surface]);
    for _ in 0..32 {
      let previous = descendants.len();
      for (id, placement) in &self.surface_placements {
        if matches!(placement.parent,LuaDrawTarget::Slice(parent) if descendants.contains(&parent))
        {
          descendants.insert(*id);
        }
      }
      if previous == descendants.len() {
        break;
      }
    }
    for binding in self.scrollboxes.values_mut() {
      if descendants.contains(&binding.surface) {
        self.ui.invalidate_pointer_scope(binding.widget.scope());
        binding.widget.deactivate();
      }
    }
    for id in descendants {
      if let Some(scope) = self.slice_scopes.remove(&id) {
        self.ui.invalidate_pointer_scope(scope);
      }
    }
  }
}
