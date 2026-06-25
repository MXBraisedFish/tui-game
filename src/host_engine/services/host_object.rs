use std::collections::HashMap;

use super::{Rect, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HostAreaKind {
  TopBar,
  Separator,
  DeveloperViewport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostArea {
  pub kind: HostAreaKind,
  pub rect: Rect,
  pub visible: bool,
}

pub struct HostObjectPool {
  areas: HashMap<HostAreaKind, HostArea>,
}

impl HostObjectPool {
  pub fn new() -> Self {
    Self {
      areas: HashMap::new(),
    }
  }

  pub fn area_rect(&self, kind: HostAreaKind) -> Option<Rect> {
    self
      .areas
      .get(&kind)
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
    self.areas.get(&kind).is_some_and(|area| area.visible)
  }

  pub(crate) fn set_area(&mut self, kind: HostAreaKind, rect: Rect, visible: bool) {
    self.areas.insert(
      kind,
      HostArea {
        kind,
        rect,
        visible,
      },
    );
  }

  pub(crate) fn clear(&mut self) {
    self.areas.clear();
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
    pool.set_area(HostAreaKind::DeveloperViewport, rect, true);
    pool.set_area(HostAreaKind::TopBar, Rect { height: 1, ..rect }, false);

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
