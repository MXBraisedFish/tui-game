//! Frame-local clipping and pointer ownership for embedded layer components.

use super::*;
use std::collections::HashSet;
use tg_service_layout::Rect;

/// The identity of an embedded pointer scope within one object pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointerScopeId(pub u64);

pub(super) struct PointerScope {
  pub parent: Option<PointerScopeId>,
  pub clip: Rect,
}

impl UiObjectPool {
  /// Allocate a pointer scope independent of user widget identities.
  pub fn allocate_pointer_scope(&mut self) -> PointerScopeId {
    self.next_pointer_scope += 1;
    PointerScopeId(self.next_pointer_scope)
  }
  /// Allocate a private canvas identity outside the user slice identifier range.
  pub fn allocate_embedded_slice(&mut self) -> tg_service_canvas::SliceId {
    self.next_embedded_slice += 1;
    tg_service_canvas::SliceId(u64::MAX - self.next_embedded_slice)
  }
  /// Enter an ancestor-clipped pointer scope; reject cycles and nesting deeper than 32.
  pub fn begin_pointer_scope(&mut self, id: PointerScopeId, clip: Rect) -> bool {
    if self.scope_stack.len() >= 32 || self.scope_stack.contains(&id) {
      return false;
    }
    self.pointer_scopes.insert(
      id,
      PointerScope {
        parent: self.scope_stack.last().copied(),
        clip,
      },
    );
    self.scope_stack.push(id);
    true
  }
  /// Leave the current pointer scope after replaying its descendants.
  pub fn end_pointer_scope(&mut self) {
    self.scope_stack.pop();
  }
  /// Register an opaque component rectangle at the current drawing position.
  pub fn register_pointer_region(&mut self, rank: usize, rect: Rect) -> u64 {
    let order = self.next_render_order();
    if rect.width > 0 && rect.height > 0 {
      self.pointer_regions.push((rect, rank, order));
    }
    order
  }
  /// Refresh root-layer occlusion after detaching embedded surfaces from global composition.
  pub fn refresh_pointer_surfaces(&self, canvas: &tg_service_canvas::CanvasService) {
    *self.pointer_surfaces.borrow_mut() = canvas.pointer_surfaces();
  }
  pub(crate) fn scope_contains_order(&self, scope: PointerScopeId, order: u64) -> bool {
    let mut current = self.order_scopes.get(&order).copied();
    for _ in 0..32 {
      let Some(id) = current else {
        return false;
      };
      if id == scope {
        return true;
      }
      current = self.pointer_scopes.get(&id).and_then(|scope| scope.parent);
    }
    false
  }
  pub(crate) fn is_pointer_region(&self, rank: usize, order: u64) -> bool {
    self
      .pointer_regions
      .iter()
      .any(|(_, current_rank, current_order)| (*current_rank, *current_order) == (rank, order))
  }
  /// Invalidate only hit geometry belonging to a removed embedded container.
  pub fn invalidate_pointer_scope(&mut self, id: PointerScopeId) {
    let orders = self
      .order_scopes
      .keys()
      .copied()
      .filter(|order| self.scope_contains_order(id, *order))
      .collect::<HashSet<_>>();
    self
      .pointer_regions
      .retain(|(_, _, order)| !orders.contains(order));
    for state in self.hit_areas.areas.values_mut() {
      if state.hit.is_some_and(|hit| orders.contains(&hit.order)) {
        state.hit = None;
      }
    }
    for state in self.hyperlinks.links.values_mut() {
      if state.hit.is_some_and(|hit| orders.contains(&hit.order)) {
        state.hit = None;
      }
    }
    self.text_inputs.invalidate_orders(&orders);
    for view in self.markdown_views.views.values_mut() {
      if view.hits.iter().any(|hit| orders.contains(&hit.order)) {
        view.pressed = None;
      }
      view.hits.retain(|hit| !orders.contains(&hit.order));
    }
  }
}
