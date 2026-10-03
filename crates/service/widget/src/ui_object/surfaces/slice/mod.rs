//! Frame-scoped rectangular surfaces with ordered composition and clipping.

use std::collections::HashMap;

use crate::SurfaceId;
use crate::UiObjectPool;
use tg_core_style::TextColor;
use tg_service_layout::{LayoutService, Rect, Size};

pub use tg_service_canvas::SliceId;

/// A fixed or source-relative size of a slice dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceLength {
  /// The fixed setting for slice length.
  Fixed(u16),
  /// The auto setting for slice length.
  Auto,
  /// The percent setting for slice length.
  Percent(u8),
}

/// Symbolic slice position and dimensions resolved against a source viewport.
///
/// # Fields
///
/// * `x` - The horizontal coordinate in terminal cells.
/// * `y` - The vertical coordinate in terminal cells.
/// * `width` - The width in terminal columns.
/// * `height` - The height in terminal rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SliceRect {
  /// The horizontal coordinate in terminal cells.
  pub x: i32,
  /// The vertical coordinate in terminal cells.
  pub y: i32,
  /// The width in terminal columns.
  pub width: SliceLength,
  /// The height in terminal rows.
  pub height: SliceLength,
}

/// Configuration values controlling slice behavior.
///
/// # Fields
///
/// * `rect` - The rectangular region in terminal cells.
/// * `visible` - Whether this surface participates in composition.
/// * `opaque` - Whether empty cells cover lower surfaces.
/// * `layer` - The layer.
/// * `background` - The background color override, or `None` to inherit the default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SliceOptions {
  /// The rectangular region in terminal cells.
  pub rect: SliceRect,
  /// Whether this surface participates in composition.
  pub visible: bool,
  /// Whether empty cells cover lower surfaces.
  pub opaque: bool,
  /// The layer.
  pub layer: Option<i32>,
  /// The background color override, or `None` to inherit the default.
  pub background: Option<TextColor>,
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
      layer: None,
      background: None,
    }
  }
}

/// The retained state of slice.
///
/// # Fields
///
/// * `rect` - The rectangular region in terminal cells.
/// * `visible` - Whether this surface participates in composition.
/// * `opaque` - Whether empty cells cover lower surfaces.
/// * `layer` - The layer.
/// * `background` - The background color override, or `None` to inherit the default.
/// * `frame_scoped` - Whether the slice must be submitted again each frame.
/// * `drawn_this_frame` - Whether this frame submitted the slice for composition.
#[derive(Clone)]
pub(crate) struct SliceState {
  /// The rectangular region in terminal cells.
  pub rect: SliceRect,
  /// Whether this surface participates in composition.
  pub visible: bool,
  /// Whether empty cells cover lower surfaces.
  pub opaque: bool,
  /// The layer.
  pub layer: i32,
  /// The background color override, or `None` to inherit the default.
  pub background: Option<TextColor>,
  /// Whether the slice must be submitted again each frame.
  pub(crate) frame_scoped: bool,
  /// Whether this frame submitted the slice for composition.
  pub(crate) drawn_this_frame: bool,
}

/// The collection of owned slice instances and their queued events.
///
/// # Fields
///
/// * `next_id` - The identifier of the next.
/// * `slices` - The slices indexed by their declared keys.
pub(crate) struct SliceObjects {
  /// The identifier of the next.
  pub next_id: u64,
  /// The slices indexed by their declared keys.
  pub slices: HashMap<SliceId, SliceState>,
}

impl SliceObjects {
  /// Create a slice objects with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      slices: HashMap::new(),
    }
  }
}

/// The public entry point for slice operations.
#[derive(Default)]
pub struct SliceService;

impl SliceService {
  /// Create a slice service with its initial state.
  pub fn new() -> Self {
    Self
  }

  /// Create an owned slice object and return its identity.
  pub fn create(&self, pool: &mut UiObjectPool, options: SliceOptions) -> Option<SliceId> {
    valid_rect(options.rect).then(|| {
      let id = SliceId(pool.slices.next_id);
      pool.slices.next_id += 1;
      let highest_layer = pool.slices.slices.len() as i32;
      let layer = options.layer.unwrap_or(highest_layer.saturating_add(1));
      let layer = layer.clamp(1, highest_layer.saturating_add(1));
      for state in pool.slices.slices.values_mut() {
        if state.layer >= layer {
          state.layer = state.layer.saturating_add(1);
        }
      }
      pool.slices.slices.insert(
        id,
        SliceState {
          rect: options.rect,
          visible: options.visible,
          opaque: options.opaque,
          layer,
          background: options.background,
          frame_scoped: false,
          drawn_this_frame: false,
        },
      );
      pool.surfaces.push(SurfaceId::Slice(id));
      reorder_slices(pool);
      id
    })
  }

  /// Remove the identified widget object and release its owned state.
  pub fn remove(&self, pool: &mut UiObjectPool, id: SliceId) -> bool {
    let Some(removed) = pool.slices.slices.remove(&id) else {
      return false;
    };
    for state in pool.slices.slices.values_mut() {
      if state.layer > removed.layer {
        state.layer -= 1;
      }
    }
    pool
      .surfaces
      .retain(|current| *current != SurfaceId::Slice(id));
    true
  }

  /// Report whether the identified widget object is still present.
  pub fn exists(&self, pool: &UiObjectPool, id: SliceId) -> bool {
    pool.slices.slices.contains_key(&id)
  }

  /// Return the original slice geometry before source-relative layout is resolved.
  pub fn configured_rect(&self, pool: &UiObjectPool, id: SliceId) -> Option<SliceRect> {
    Some(pool.slices.slices.get(&id)?.rect)
  }

  /// Return the component's rectangle after symbolic coordinates and source dimensions are
  /// resolved.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn resolved_rect(
    &self,
    pool: &UiObjectPool,
    id: SliceId,
    layout: &LayoutService,
  ) -> Option<Rect> {
    Some(resolve_rect(pool.slices.slices.get(&id)?.rect, layout))
  }

  /// Return the component's resolved terminal width and height.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn resolved_size(
    &self,
    pool: &UiObjectPool,
    id: SliceId,
    layout: &LayoutService,
  ) -> Option<Size> {
    let rect = self.resolved_rect(pool, id, layout)?;
    Some(Size {
      width: rect.width,
      height: rect.height,
    })
  }

  /// Return the component's resolved width in terminal columns.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn resolved_width(
    &self,
    pool: &UiObjectPool,
    id: SliceId,
    layout: &LayoutService,
  ) -> Option<u16> {
    Some(self.resolved_size(pool, id, layout)?.width)
  }

  /// Return the component's resolved height in terminal rows.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn resolved_height(
    &self,
    pool: &UiObjectPool,
    id: SliceId,
    layout: &LayoutService,
  ) -> Option<u16> {
    Some(self.resolved_size(pool, id, layout)?.height)
  }

  /// Update the rect used by this slice service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `rect` - The rectangular region in terminal cells.
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

  /// Update the position used by this slice service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `x` - The horizontal coordinate in terminal cells.
  /// * `y` - The vertical coordinate in terminal cells.
  pub fn set_position(&self, pool: &mut UiObjectPool, id: SliceId, x: i32, y: i32) -> bool {
    let Some(state) = pool.slices.slices.get_mut(&id) else {
      return false;
    };
    state.rect.x = x;
    state.rect.y = y;
    true
  }

  /// Set whether the slice must be submitted again for each host frame.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `frame_scoped` - The frame scoped.
  pub fn set_frame_scoped(&self, pool: &mut UiObjectPool, id: SliceId, frame_scoped: bool) -> bool {
    let Some(state) = pool.slices.slices.get_mut(&id) else {
      return false;
    };
    state.frame_scoped = frame_scoped;
    state.drawn_this_frame = false;
    true
  }

  /// Submit the component's configured drawing state to the current canvas frame.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `x` - The horizontal coordinate in terminal cells.
  /// * `y` - The vertical coordinate in terminal cells.
  pub fn draw(&self, pool: &mut UiObjectPool, id: SliceId, x: i32, y: i32) -> bool {
    let Some(state) = pool.slices.slices.get_mut(&id) else {
      return false;
    };
    state.rect.x = x;
    state.rect.y = y;
    state.drawn_this_frame = true;
    true
  }

  /// Prepare per-frame state and discard submissions or observations belonging to the previous
  /// frame.
  pub fn begin_frame(&self, pool: &mut UiObjectPool) {
    for state in pool.slices.slices.values_mut() {
      if state.frame_scoped {
        state.drawn_this_frame = false;
      }
    }
  }

  /// Return the layer for the addressed object when it is available.
  pub fn layer(&self, pool: &UiObjectPool, id: SliceId) -> Option<i32> {
    Some(pool.slices.slices.get(&id)?.layer)
  }

  /// Update the layer used by this slice service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layer` - The layer.
  pub fn set_layer(&self, pool: &mut UiObjectPool, id: SliceId, layer: i32) -> bool {
    let Some(old_layer) = pool.slices.slices.get(&id).map(|state| state.layer) else {
      return false;
    };
    let layer = layer.clamp(1, pool.slices.slices.len() as i32);
    if layer < old_layer {
      for (current_id, state) in &mut pool.slices.slices {
        if *current_id != id && state.layer >= layer && state.layer < old_layer {
          state.layer += 1;
        }
      }
    } else if layer > old_layer {
      for (current_id, state) in &mut pool.slices.slices {
        if *current_id != id && state.layer > old_layer && state.layer <= layer {
          state.layer -= 1;
        }
      }
    }
    pool.slices.slices.get_mut(&id).unwrap().layer = layer;
    reorder_slices(pool);
    true
  }

  /// Return the background for the addressed object when it is available.
  pub fn background(&self, pool: &UiObjectPool, id: SliceId) -> Option<Option<TextColor>> {
    Some(pool.slices.slices.get(&id)?.background.clone())
  }

  /// Assign the cell background color.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `background` - The cell background color.
  pub fn set_background(
    &self,
    pool: &mut UiObjectPool,
    id: SliceId,
    background: Option<TextColor>,
  ) -> bool {
    let Some(state) = pool.slices.slices.get_mut(&id) else {
      return false;
    };
    state.background = background;
    true
  }

  /// Return the ids for the addressed object.
  pub fn ids(&self, pool: &UiObjectPool) -> Vec<SliceId> {
    let mut ids = pool.slices.slices.keys().copied().collect::<Vec<_>>();
    ids.sort_by_key(|id| id.0);
    ids
  }

  /// Return visible slice identifiers in their resolved composition order for the requested
  /// layer.
  pub fn ids_by_layer(&self, pool: &UiObjectPool) -> Vec<SliceId> {
    let mut ids = self.ids(pool);
    ids.sort_by_key(|id| (pool.slices.slices[id].layer, id.0));
    ids
  }

  /// Report whether the addressed object is visible.
  pub fn is_visible(&self, pool: &UiObjectPool, id: SliceId) -> bool {
    pool
      .slices
      .slices
      .get(&id)
      .is_some_and(|state| state.visible && (!state.frame_scoped || state.drawn_this_frame))
  }

  /// Update the visible used by this slice service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `visible` - Whether the target contributes to the visible frame.
  pub fn set_visible(&self, pool: &mut UiObjectPool, id: SliceId, visible: bool) -> bool {
    let Some(state) = pool.slices.slices.get_mut(&id) else {
      return false;
    };
    state.visible = visible;
    true
  }

  /// Report whether the addressed object is opaque.
  pub fn is_opaque(&self, pool: &UiObjectPool, id: SliceId) -> bool {
    pool
      .slices
      .slices
      .get(&id)
      .is_some_and(|state| state.opaque)
  }

  /// Move the surface to the front of its composition group.
  pub fn bring_to_front(&self, pool: &mut UiObjectPool, id: SliceId) -> bool {
    pool.move_surface_to_edge(SurfaceId::Slice(id), false)
  }

  /// Move the surface to the back of its composition group.
  pub fn send_to_back(&self, pool: &mut UiObjectPool, id: SliceId) -> bool {
    pool.move_surface_to_edge(SurfaceId::Slice(id), true)
  }

  /// Place the surface immediately above the referenced peer.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `target` - The object or resource affected by the operation.
  pub fn move_above(&self, pool: &mut UiObjectPool, id: SliceId, target: SliceId) -> bool {
    pool.move_surface_relative(SurfaceId::Slice(id), SurfaceId::Slice(target), true)
  }

  /// Place the surface immediately below the referenced peer.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `target` - The object or resource affected by the operation.
  pub fn move_below(&self, pool: &mut UiObjectPool, id: SliceId, target: SliceId) -> bool {
    pool.move_surface_relative(SurfaceId::Slice(id), SurfaceId::Slice(target), false)
  }
}

/// Resolve a rectangular request against its coordinate space and available bounds.
pub(crate) fn resolve_rect(rect: SliceRect, layout: &LayoutService) -> Rect {
  resolve_rect_with_source(rect, layout.developer_size()).0
}

/// Resolve slice geometry against the explicitly supplied source dimensions.
pub(crate) fn resolve_rect_with_source(rect: SliceRect, viewport: Size) -> (Rect, u16, u16) {
  let resolve = |length: SliceLength, total: u16, offset: i32| match length {
    SliceLength::Fixed(value) => i64::from(value),
    SliceLength::Auto => (i64::from(total) - i64::from(offset.max(0))).max(0),
    SliceLength::Percent(value) => i64::from((total as u32 * value as u32 / 100) as u16),
  };
  let clip_axis = |offset: i32, length: i64, total: u16| {
    let start = i64::from(offset);
    let end = start.saturating_add(length);
    let visible_start = start.clamp(0, i64::from(total));
    let visible_end = end.clamp(0, i64::from(total));
    (
      visible_start as u16,
      visible_end.saturating_sub(visible_start) as u16,
      visible_start.saturating_sub(start).clamp(0, length.max(0)) as u16,
    )
  };
  let (x, width, source_x) = clip_axis(
    rect.x,
    resolve(rect.width, viewport.width, rect.x),
    viewport.width,
  );
  let (y, height, source_y) = clip_axis(
    rect.y,
    resolve(rect.height, viewport.height, rect.y),
    viewport.height,
  );
  (
    Rect {
      x,
      y,
      width,
      height,
    },
    source_x,
    source_y,
  )
}

fn valid_rect(rect: SliceRect) -> bool {
  let valid = |length| !matches!(length, SliceLength::Percent(value) if value > 100);
  valid(rect.width) && valid(rect.height)
}

fn reorder_slices(pool: &mut UiObjectPool) {
  let mut ordered = pool.slices.slices.keys().copied().collect::<Vec<_>>();
  ordered.sort_by_key(|id| (pool.slices.slices[id].layer, id.0));
  let mut ordered = ordered.into_iter();
  for surface in &mut pool.surfaces {
    if matches!(surface, SurfaceId::Slice(_))
      && let Some(id) = ordered.next()
    {
      *surface = SurfaceId::Slice(id);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn rect(x: i32, y: i32, width: SliceLength, height: SliceLength) -> SliceRect {
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
          ..Default::default()
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
    assert_eq!(
      service.configured_rect(&pool, a),
      Some(rect(10, 5, SliceLength::Percent(50), SliceLength::Auto))
    );
    assert_eq!(
      service.resolved_size(&pool, a, &layout),
      Some(Size {
        width: 50,
        height: 35
      })
    );
    assert_eq!(service.resolved_width(&pool, a, &layout), Some(50));
    assert_eq!(service.resolved_height(&pool, a, &layout), Some(35));
    assert!(!service.is_visible(&pool, b));
    assert!(!service.is_opaque(&pool, b));
    assert!(service.send_to_back(&mut pool, b));
    assert_eq!(
      pool.surfaces,
      vec![SurfaceId::Slice(b), SurfaceId::Slice(a)]
    );
    assert!(service.move_above(&mut pool, b, a));
    assert_eq!(
      pool.surfaces,
      vec![SurfaceId::Slice(a), SurfaceId::Slice(b)]
    );
    assert!(service.move_below(&mut pool, b, a));
    assert_eq!(
      pool.surfaces,
      vec![SurfaceId::Slice(b), SurfaceId::Slice(a)]
    );
    assert!(service.bring_to_front(&mut pool, b));
    assert_eq!(
      pool.surfaces,
      vec![SurfaceId::Slice(a), SurfaceId::Slice(b)]
    );
    assert!(!service.move_above(&mut pool, a, a));
    assert!(service.remove(&mut pool, a));
    assert!(!service.exists(&pool, a));
  }

  #[test]
  fn frame_scoped_slice_requires_draw_on_every_frame() {
    let service = SliceService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        SliceOptions {
          rect: rect(0, 0, SliceLength::Fixed(4), SliceLength::Fixed(2)),
          ..Default::default()
        },
      )
      .unwrap();

    assert!(service.is_visible(&pool, id));
    assert!(service.set_frame_scoped(&mut pool, id, true));
    assert!(!service.is_visible(&pool, id));

    assert!(service.set_position(&mut pool, id, 3, 4));
    assert!(!service.is_visible(&pool, id));
    assert!(service.draw(&mut pool, id, 5, 6));
    assert!(service.is_visible(&pool, id));
    assert_eq!(service.configured_rect(&pool, id).unwrap().x, 5);
    assert_eq!(service.configured_rect(&pool, id).unwrap().y, 6);

    service.begin_frame(&mut pool);
    assert!(!service.is_visible(&pool, id));
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

  #[test]
  fn public_layers_remain_contiguous_when_inserted_moved_and_removed() {
    let service = SliceService::new();
    let mut pool = UiObjectPool::new();
    let options = |layer| SliceOptions {
      rect: rect(0, 0, SliceLength::Fixed(2), SliceLength::Fixed(2)),
      layer,
      ..Default::default()
    };

    let first = service.create(&mut pool, options(None)).unwrap();
    let second = service.create(&mut pool, options(None)).unwrap();
    let inserted = service.create(&mut pool, options(Some(1))).unwrap();
    assert_eq!(service.layer(&pool, inserted), Some(1));
    assert_eq!(service.layer(&pool, first), Some(2));
    assert_eq!(service.layer(&pool, second), Some(3));
    assert_eq!(service.ids_by_layer(&pool), vec![inserted, first, second]);

    assert!(service.set_layer(&mut pool, second, 1));
    assert_eq!(service.ids_by_layer(&pool), vec![second, inserted, first]);
    assert_eq!(service.layer(&pool, second), Some(1));
    assert_eq!(service.layer(&pool, inserted), Some(2));
    assert_eq!(service.layer(&pool, first), Some(3));

    assert!(service.set_layer(&mut pool, second, i32::MAX));
    assert_eq!(service.ids_by_layer(&pool), vec![inserted, first, second]);
    assert_eq!(service.layer(&pool, second), Some(3));

    assert!(service.remove(&mut pool, first));
    assert_eq!(service.ids_by_layer(&pool), vec![inserted, second]);
    assert_eq!(service.layer(&pool, inserted), Some(1));
    assert_eq!(service.layer(&pool, second), Some(2));
  }

  #[test]
  fn background_can_be_set_cleared_and_queried() {
    let service = SliceService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        SliceOptions {
          background: Some(TextColor::Terminal(tg_core_style::TerminalColor::Blue)),
          ..Default::default()
        },
      )
      .unwrap();
    assert_eq!(
      service.background(&pool, id),
      Some(Some(TextColor::Terminal(
        tg_core_style::TerminalColor::Blue,
      )))
    );
    assert!(service.set_background(&mut pool, id, None));
    assert_eq!(service.background(&pool, id), Some(None));
    assert!(!service.set_background(&mut pool, SliceId(999), None));
  }

  #[test]
  fn negative_and_far_positive_positions_are_clipped_without_wrapping() {
    let service = SliceService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);

    let partially_visible = service
      .create(
        &mut pool,
        SliceOptions {
          rect: rect(-4, -2, SliceLength::Fixed(10), SliceLength::Auto),
          ..Default::default()
        },
      )
      .unwrap();
    assert_eq!(
      service.resolved_rect(&pool, partially_visible, &layout),
      Some(Rect {
        x: 0,
        y: 0,
        width: 6,
        height: 8,
      })
    );
    assert_eq!(
      resolve_rect_with_source(
        rect(-4, -2, SliceLength::Fixed(10), SliceLength::Auto),
        layout.developer_size(),
      ),
      (
        Rect {
          x: 0,
          y: 0,
          width: 6,
          height: 8,
        },
        4,
        2,
      )
    );

    let outside = service
      .create(
        &mut pool,
        SliceOptions {
          rect: rect(i32::MAX, i32::MAX, SliceLength::Auto, SliceLength::Auto),
          ..Default::default()
        },
      )
      .unwrap();
    assert_eq!(
      service.resolved_rect(&pool, outside, &layout),
      Some(Rect {
        x: 20,
        y: 10,
        width: 0,
        height: 0,
      })
    );
  }
}
