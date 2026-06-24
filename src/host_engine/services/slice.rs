use std::collections::HashMap;

use super::ui::UiObjectPool;
use super::{LayoutService, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SliceId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceLength {
  Fixed(u16),
  Auto,
  Percent(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SliceRect {
  pub x: u16,
  pub y: u16,
  pub width: SliceLength,
  pub height: SliceLength,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SliceOptions {
  pub rect: SliceRect,
  pub visible: bool,
  pub opaque: bool,
}

impl Default for SliceOptions {
  fn default() -> Self {
    Self {
      rect: SliceRect {
        x: 0,
        y: 0,
        width: SliceLength::Auto,
        height: SliceLength::Auto,
      },
      visible: true,
      opaque: true,
    }
  }
}

#[derive(Clone, Copy)]
pub(crate) struct SliceState {
  pub rect: SliceRect,
  pub visible: bool,
  pub opaque: bool,
}

pub(crate) struct SliceObjects {
  pub next_id: u64,
  pub slices: HashMap<SliceId, SliceState>,
  pub order: Vec<SliceId>,
}

impl SliceObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      slices: HashMap::new(),
      order: Vec::new(),
    }
  }
}

pub struct SliceService;

impl SliceService {
  pub fn new() -> Self {
    Self
  }

  pub fn create(&self, pool: &mut UiObjectPool, options: SliceOptions) -> Option<SliceId> {
    valid_rect(options.rect).then(|| {
      let id = SliceId(pool.slices.next_id);
      pool.slices.next_id += 1;
      pool.slices.slices.insert(
        id,
        SliceState {
          rect: options.rect,
          visible: options.visible,
          opaque: options.opaque,
        },
      );
      pool.slices.order.push(id);
      id
    })
  }

  pub fn remove(&self, pool: &mut UiObjectPool, id: SliceId) -> bool {
    if pool.slices.slices.remove(&id).is_none() {
      return false;
    }
    pool.slices.order.retain(|current| *current != id);
    true
  }

  pub fn exists(&self, pool: &UiObjectPool, id: SliceId) -> bool {
    pool.slices.slices.contains_key(&id)
  }

  pub fn rect(&self, pool: &UiObjectPool, id: SliceId) -> Option<SliceRect> {
    Some(pool.slices.slices.get(&id)?.rect)
  }

  pub fn resolved_rect(
    &self,
    pool: &UiObjectPool,
    id: SliceId,
    layout: &LayoutService,
  ) -> Option<Rect> {
    Some(resolve_rect(pool.slices.slices.get(&id)?.rect, layout))
  }

  pub fn set_rect(&self, pool: &mut UiObjectPool, id: SliceId, rect: SliceRect) -> bool {
    if !valid_rect(rect) {
      return false;
    }
    let Some(state) = pool.slices.slices.get_mut(&id) else {
      return false;
    };
    state.rect = rect;
    true
  }

  pub fn is_visible(&self, pool: &UiObjectPool, id: SliceId) -> bool {
    pool
      .slices
      .slices
      .get(&id)
      .is_some_and(|state| state.visible)
  }

  pub fn set_visible(&self, pool: &mut UiObjectPool, id: SliceId, visible: bool) -> bool {
    let Some(state) = pool.slices.slices.get_mut(&id) else {
      return false;
    };
    state.visible = visible;
    true
  }

  pub fn is_opaque(&self, pool: &UiObjectPool, id: SliceId) -> bool {
    pool
      .slices
      .slices
      .get(&id)
      .is_some_and(|state| state.opaque)
  }

  pub fn bring_to_front(&self, pool: &mut UiObjectPool, id: SliceId) -> bool {
    move_to_edge(&mut pool.slices, id, false)
  }

  pub fn send_to_back(&self, pool: &mut UiObjectPool, id: SliceId) -> bool {
    move_to_edge(&mut pool.slices, id, true)
  }

  pub fn move_above(&self, pool: &mut UiObjectPool, id: SliceId, target: SliceId) -> bool {
    move_relative(&mut pool.slices, id, target, true)
  }

  pub fn move_below(&self, pool: &mut UiObjectPool, id: SliceId, target: SliceId) -> bool {
    move_relative(&mut pool.slices, id, target, false)
  }
}

pub(crate) fn resolve_rect(rect: SliceRect, layout: &LayoutService) -> Rect {
  let viewport = layout.viewport_size();
  let x = rect.x.min(viewport.width);
  let y = rect.y.min(viewport.height);
  let resolve = |length: SliceLength, total: u16, offset: u16| match length {
    SliceLength::Fixed(value) => value,
    SliceLength::Auto => total.saturating_sub(offset),
    SliceLength::Percent(value) => (total as u32 * value as u32 / 100) as u16,
  };
  Rect {
    x,
    y,
    width: resolve(rect.width, viewport.width, x).min(viewport.width.saturating_sub(x)),
    height: resolve(rect.height, viewport.height, y).min(viewport.height.saturating_sub(y)),
  }
}

fn valid_rect(rect: SliceRect) -> bool {
  let valid = |length| !matches!(length, SliceLength::Percent(value) if value > 100);
  valid(rect.width) && valid(rect.height)
}

fn move_to_edge(objects: &mut SliceObjects, id: SliceId, back: bool) -> bool {
  let Some(index) = objects.order.iter().position(|current| *current == id) else {
    return false;
  };
  objects.order.remove(index);
  if back {
    objects.order.insert(0, id);
  } else {
    objects.order.push(id);
  }
  true
}

fn move_relative(objects: &mut SliceObjects, id: SliceId, target: SliceId, above: bool) -> bool {
  if id == target || !objects.slices.contains_key(&id) || !objects.slices.contains_key(&target) {
    return false;
  }
  objects.order.retain(|current| *current != id);
  let target_index = objects
    .order
    .iter()
    .position(|current| *current == target)
    .unwrap();
  objects.order.insert(target_index + usize::from(above), id);
  true
}

#[cfg(test)]
mod tests {
  use super::*;

  fn rect(x: u16, y: u16, width: SliceLength, height: SliceLength) -> SliceRect {
    SliceRect {
      x,
      y,
      width,
      height,
    }
  }

  #[test]
  fn lifecycle_resolution_and_order_are_stable() {
    let service = SliceService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(100, 40);
    let a = service
      .create(
        &mut pool,
        SliceOptions {
          rect: rect(10, 5, SliceLength::Percent(50), SliceLength::Auto),
          ..Default::default()
        },
      )
      .unwrap();
    let b = service
      .create(
        &mut pool,
        SliceOptions {
          rect: rect(0, 0, SliceLength::Fixed(20), SliceLength::Fixed(10)),
          visible: false,
          opaque: false,
        },
      )
      .unwrap();
    assert_eq!((a, b), (SliceId(1), SliceId(2)));
    assert_eq!(
      service.resolved_rect(&pool, a, &layout),
      Some(Rect {
        x: 10,
        y: 5,
        width: 50,
        height: 35
      })
    );
    assert!(!service.is_visible(&pool, b));
    assert!(!service.is_opaque(&pool, b));
    assert!(service.send_to_back(&mut pool, b));
    assert_eq!(pool.slices.order, vec![b, a]);
    assert!(service.move_above(&mut pool, b, a));
    assert_eq!(pool.slices.order, vec![a, b]);
    assert!(service.move_below(&mut pool, b, a));
    assert_eq!(pool.slices.order, vec![b, a]);
    assert!(service.bring_to_front(&mut pool, b));
    assert_eq!(pool.slices.order, vec![a, b]);
    assert!(!service.move_above(&mut pool, a, a));
    assert!(service.remove(&mut pool, a));
    assert!(!service.exists(&pool, a));
  }

  #[test]
  fn invalid_percent_is_rejected_and_rect_is_clipped() {
    let service = SliceService::new();
    let mut pool = UiObjectPool::new();
    assert!(
      service
        .create(
          &mut pool,
          SliceOptions {
            rect: rect(0, 0, SliceLength::Percent(101), SliceLength::Auto),
            ..Default::default()
          }
        )
        .is_none()
    );
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    let id = service
      .create(
        &mut pool,
        SliceOptions {
          rect: rect(18, 8, SliceLength::Fixed(20), SliceLength::Percent(100)),
          ..Default::default()
        },
      )
      .unwrap();
    assert_eq!(
      service.resolved_rect(&pool, id, &layout),
      Some(Rect {
        x: 18,
        y: 8,
        width: 2,
        height: 2
      })
    );
  }
}
