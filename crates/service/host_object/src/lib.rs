//! Host UI drawing areas backed by reusable slice identifiers.
//!
//! # Examples
//!
//! ```rust
//! use tg_core_geometry::Rect;
//! use tg_service_host_object::{HostAreaKind, HostObjectPool};
//!
//! fn main() {
//!   let mut pool = HostObjectPool::new();
//!   let top = pool.ensure_area(HostAreaKind::TopBar);
//!   let rect = Rect {
//!     x: 0,
//!     y: 0,
//!     width: 80,
//!     height: 1,
//!   };
//!   assert!(pool.update_area(top, rect, true));
//!   assert_eq!(pool.area_width(HostAreaKind::TopBar), Some(80));
//!   assert!(pool.is_visible(HostAreaKind::TopBar));
//!   println!("host_object ok: {:?}", pool.area_rect(HostAreaKind::TopBar));
//! }
//! ```

use std::collections::HashMap;

use tg_core_geometry::{Rect, Size};

/// The identity of host area within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HostAreaId(
  /// The wrapped u64 value.
  pub u64,
);

/// The host UI area whose slice identity is managed by the area pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HostAreaKind {
  /// The top bar setting for host area kind.
  TopBar,
  /// The separator setting for host area kind.
  Separator,
  /// The developer viewport setting for host area kind.
  DeveloperViewport,
}

/// The host area representation used by this module.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `kind` - The host area kind carried by this host area.
/// * `rect` - The rectangular region in terminal cells.
/// * `visible` - Whether this surface participates in composition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostArea {
  /// The identifier of the owned object.
  pub id: HostAreaId,
  /// The host area kind carried by this host area.
  pub kind: HostAreaKind,
  /// The rectangular region in terminal cells.
  pub rect: Rect,
  /// Whether this surface participates in composition.
  pub visible: bool,
}

/// Owned host object instances with live-identifier lookup and removal.
pub struct HostObjectPool {
  next_area_id: u64,
  areas: HashMap<HostAreaId, HostArea>,
  areas_by_kind: HashMap<HostAreaKind, HostAreaId>,
}

impl HostObjectPool {
  /// Create a host object pool with its initial state.
  pub fn new() -> Self {
    Self {
      next_area_id: 1,
      areas: HashMap::new(),
      areas_by_kind: HashMap::new(),
    }
  }

  /// Register a host UI area with configured bounds, returning `None` when its kind is already
  /// registered.
  ///
  /// # Arguments
  ///
  /// * `kind` - The requested event, object, or path kind.
  /// * `rect` - The rectangular region in terminal cells.
  /// * `visible` - Whether the target contributes to the visible frame.
  pub fn create_area(
    &mut self,
    kind: HostAreaKind,
    rect: Rect,
    visible: bool,
  ) -> Option<HostAreaId> {
    if self.areas_by_kind.contains_key(&kind) {
      return None;
    }
    let id = HostAreaId(self.next_area_id);
    self.next_area_id += 1;
    self.areas.insert(
      id,
      HostArea {
        id,
        kind,
        rect,
        visible,
      },
    );
    self.areas_by_kind.insert(kind, id);
    Some(id)
  }

  /// Return the identity registered for the requested host area kind.
  pub fn area_id(&self, kind: HostAreaKind) -> Option<HostAreaId> {
    self.areas_by_kind.get(&kind).copied()
  }

  /// Return the configured terminal-cell rectangle for the requested host area.
  pub fn area_rect(&self, kind: HostAreaKind) -> Option<Rect> {
    self
      .area_by_kind(kind)
      .filter(|area| area.visible)
      .map(|area| area.rect)
  }

  /// Return the registered host area's width and height in terminal cells.
  pub fn area_size(&self, kind: HostAreaKind) -> Option<Size> {
    let rect = self.area_rect(kind)?;
    Some(Size {
      width: rect.width,
      height: rect.height,
    })
  }

  /// Return the registered host area's width in terminal columns.
  pub fn area_width(&self, kind: HostAreaKind) -> Option<u16> {
    Some(self.area_size(kind)?.width)
  }

  /// Return the registered host area's height in terminal rows.
  pub fn area_height(&self, kind: HostAreaKind) -> Option<u16> {
    Some(self.area_size(kind)?.height)
  }

  /// Report whether the addressed object is visible.
  pub fn is_visible(&self, kind: HostAreaKind) -> bool {
    self.area_by_kind(kind).is_some_and(|area| area.visible)
  }

  /// Return the existing host area identity or register a hidden area with default bounds.
  ///
  /// # Panics
  ///
  /// Panic if an internal invariant is violated: `host area kind should be unique`.
  pub fn ensure_area(&mut self, kind: HostAreaKind) -> HostAreaId {
    if let Some(id) = self.area_id(kind) {
      return id;
    }
    self
      .create_area(kind, Rect::default(), false)
      .expect("host area kind should be unique")
  }

  /// Update the registered host area's rectangle and visibility, returning whether it exists.
  ///
  /// # Arguments
  ///
  /// * `id` - The identifier of the owned object.
  /// * `rect` - The rectangular region in terminal cells.
  /// * `visible` - Whether the target contributes to the visible frame.
  pub fn update_area(&mut self, id: HostAreaId, rect: Rect, visible: bool) -> bool {
    let Some(area) = self.areas.get_mut(&id) else {
      return false;
    };
    area.rect = rect;
    area.visible = visible;
    true
  }

  /// Remove all registered host UI areas and their lookup state.
  pub fn clear(&mut self) {
    self.areas.clear();
    self.areas_by_kind.clear();
  }

  fn area_by_kind(&self, kind: HostAreaKind) -> Option<&HostArea> {
    self.area_id(kind).and_then(|id| self.areas.get(&id))
  }
}

impl Default for HostObjectPool {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn host_area_queries_ignore_invisible_areas() {
    let mut pool = HostObjectPool::new();
    let rect = Rect {
      x: 0,
      y: 2,
      width: 80,
      height: 22,
    };
    assert_eq!(
      pool.create_area(HostAreaKind::DeveloperViewport, rect, true),
      Some(HostAreaId(1))
    );
    assert_eq!(
      pool.create_area(HostAreaKind::DeveloperViewport, rect, true),
      None
    );
    let top = pool.ensure_area(HostAreaKind::TopBar);
    assert!(pool.update_area(top, Rect { height: 1, ..rect }, false));

    assert_eq!(pool.area_rect(HostAreaKind::DeveloperViewport), Some(rect));
    assert_eq!(
      pool.area_size(HostAreaKind::DeveloperViewport),
      Some(Size {
        width: 80,
        height: 22
      })
    );
    assert_eq!(pool.area_rect(HostAreaKind::TopBar), None);
    assert!(!pool.is_visible(HostAreaKind::TopBar));
  }
}
