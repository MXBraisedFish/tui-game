use std::collections::HashMap;

use super::{Rect, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HostAreaId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HostAreaKind {
  TopBar,
  Separator,
  DeveloperViewport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostArea {
  pub id: HostAreaId,
  pub kind: HostAreaKind,
  pub rect: Rect,
  pub visible: bool,
}

pub struct HostObjectPool {
  next_area_id: u64,
  areas: HashMap<HostAreaId, HostArea>,
  areas_by_kind: HashMap<HostAreaKind, HostAreaId>,
}

impl HostObjectPool {
  pub fn new() -> Self {
    Self {
      next_area_id: 1,
      areas: HashMap::new(),
      areas_by_kind: HashMap::new(),
    }
  }

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

  pub fn area_id(&self, kind: HostAreaKind) -> Option<HostAreaId> {
    self.areas_by_kind.get(&kind).copied()
  }

  pub fn area_rect(&self, kind: HostAreaKind) -> Option<Rect> {
    self
      .area_by_kind(kind)
      .filter(|area| area.visible)
      .map(|area| area.rect)
  }

  pub fn area_size(&self, kind: HostAreaKind) -> Option<Size> {
    let rect = self.area_rect(kind)?;
    Some(Size {
      width: rect.width,
      height: rect.height,
    })
  }

  pub fn area_width(&self, kind: HostAreaKind) -> Option<u16> {
    Some(self.area_size(kind)?.width)
  }

  pub fn area_height(&self, kind: HostAreaKind) -> Option<u16> {
    Some(self.area_size(kind)?.height)
  }

  pub fn is_visible(&self, kind: HostAreaKind) -> bool {
    self.area_by_kind(kind).is_some_and(|area| area.visible)
  }

  pub(crate) fn ensure_area(&mut self, kind: HostAreaKind) -> HostAreaId {
    if let Some(id) = self.area_id(kind) {
      return id;
    }
    self
      .create_area(kind, Rect::default(), false)
      .expect("host area kind should be unique")
  }

  pub(crate) fn update_area(&mut self, id: HostAreaId, rect: Rect, visible: bool) -> bool {
    let Some(area) = self.areas.get_mut(&id) else {
      return false;
    };
    area.rect = rect;
    area.visible = visible;
    true
  }

  pub(crate) fn clear(&mut self) {
    self.areas.clear();
    self.areas_by_kind.clear();
  }

  fn area_by_kind(&self, kind: HostAreaKind) -> Option<&HostArea> {
    self.area_id(kind).and_then(|id| self.areas.get(&id))
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
