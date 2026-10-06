//! Clipped content viewports, scrollbar layout, and anchored pointer dragging.

mod state;
mod types;

pub(crate) use self::state::ScrollBoxObjects;
use self::state::{ScrollBoxDragState, ScrollBoxState};
use self::types::ScrollbarAxis;
pub use self::types::{
  Overflow, ScrollBoxEvent, ScrollBoxId, ScrollBoxOptions, ScrollbarLayout, ScrollbarPolicy,
  ScrollbarSide, ScrollbarStyle, ScrollbarVisibility,
};
use super::surface::SurfaceId;
use crate::UiObjectPool;
use tg_core_input::{MouseEvent, MouseEventKind, ScrollDirection};
use tg_core_unicode::char_width;
use tg_service_canvas::CanvasService;
pub(crate) use tg_service_canvas::ResolvedScrollBoxLayout;
use tg_service_layout::{LayoutService, Rect, Size};

/// The public entry point for scroll box operations.
#[derive(Default)]
pub struct ScrollBoxService;

impl ScrollBoxService {
  /// Create a scroll box service with its initial state.
  pub fn new() -> Self {
    Self
  }

  /// Create an owned scroll box object and return its identity.
  pub fn create(
    &self,
    pool: &mut UiObjectPool,
    mut options: ScrollBoxOptions,
  ) -> Option<ScrollBoxId> {
    validate_scrollbar_chars(&mut options);
    valid_options(&options).then(|| {
      let id = ScrollBoxId(pool.scroll_boxes.next_id);
      pool.scroll_boxes.next_id += 1;
      pool.scroll_boxes.boxes.insert(
        id,
        ScrollBoxState {
          options,
          scroll_x: 0,
          scroll_y: 0,
        },
      );
      pool.surfaces.push(SurfaceId::ScrollBox(id));
      id
    })
  }

  /// Remove the identified widget object and release its owned state.
  pub fn remove(&self, pool: &mut UiObjectPool, id: ScrollBoxId) -> bool {
    if pool.scroll_boxes.boxes.remove(&id).is_none() {
      return false;
    }
    pool
      .surfaces
      .retain(|surface| *surface != SurfaceId::ScrollBox(id));
    true
  }

  /// Report whether the identified widget object is still present.
  pub fn exists(&self, pool: &UiObjectPool, id: ScrollBoxId) -> bool {
    pool.scroll_boxes.boxes.contains_key(&id)
  }

  /// Return the rect for the addressed object when it is available.
  pub fn rect(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<Rect> {
    Some(pool.scroll_boxes.boxes.get(&id)?.options.rect)
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
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<Rect> {
    let rect = self.rect(pool, id)?;
    Some(clamp_rect(rect, layout.developer_size()))
  }

  /// Return the viewport size for the addressed object when it is available.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn viewport_size(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<Size> {
    let rect = self.resolved_rect(pool, id, layout)?;
    Some(Size {
      width: rect.width,
      height: rect.height,
    })
  }

  /// Return the viewport width in terminal columns for the addressed object when it is available.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn viewport_width(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<u16> {
    Some(self.viewport_size(pool, id, layout)?.width)
  }

  /// Return the viewport height in terminal rows for the addressed object when it is available.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn viewport_height(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<u16> {
    Some(self.viewport_size(pool, id, layout)?.height)
  }

  /// Return the content viewport size after inside or reserved scrollbar space is deducted.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn effective_viewport_size(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<Size> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    Some(effective_viewport(state, layout.developer_size()))
  }

  /// Return the component rectangle including any externally placed scrollbars.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn occupied_rect(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<Rect> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    Some(resolve_scroll_box_layout(state, layout.developer_size()).occupied_rect)
  }

  /// Return the terminal rectangle in which content can actually be displayed and hit-tested.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn content_viewport_rect(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<Rect> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    Some(resolve_scroll_box_layout(state, layout.developer_size()).content_viewport_rect)
  }

  /// Update the rect used by this scroll box service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `rect` - The rectangular region in terminal cells.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn set_rect(
    &self,
    pool: &mut UiObjectPool,
    id: ScrollBoxId,
    rect: Rect,
    layout: &LayoutService,
  ) -> bool {
    let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
      return false;
    };
    state.options.rect = rect;
    clamp_scroll(state, layout.developer_size());
    true
  }

  /// Return the content size for the addressed object when it is available.
  pub fn content_size(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<Size> {
    let options = &pool.scroll_boxes.boxes.get(&id)?.options;
    Some(Size {
      width: options.content_width,
      height: options.content_height,
    })
  }

  /// Update the content size used by this scroll box service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `width` - The width in terminal columns.
  /// * `height` - The height in terminal rows.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn set_content_size(
    &self,
    pool: &mut UiObjectPool,
    id: ScrollBoxId,
    width: u16,
    height: u16,
    layout: &LayoutService,
  ) -> bool {
    let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
      return false;
    };
    state.options.content_width = width;
    state.options.content_height = height;
    clamp_scroll(state, layout.developer_size());
    true
  }

  /// Report whether the addressed object is visible.
  pub fn is_visible(&self, pool: &UiObjectPool, id: ScrollBoxId) -> bool {
    pool
      .scroll_boxes
      .boxes
      .get(&id)
      .is_some_and(|state| state.options.visible)
  }

  /// Update the visible used by this scroll box service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `visible` - Whether the target contributes to the visible frame.
  pub fn set_visible(&self, pool: &mut UiObjectPool, id: ScrollBoxId, visible: bool) -> bool {
    let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
      return false;
    };
    state.options.visible = visible;
    true
  }

  /// Report whether the addressed object is opaque.
  pub fn is_opaque(&self, pool: &UiObjectPool, id: ScrollBoxId) -> bool {
    pool
      .scroll_boxes
      .boxes
      .get(&id)
      .is_some_and(|state| state.options.opaque)
  }

  /// Update the opaque used by this scroll box service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `opaque` - Whether the surface background replaces content below it.
  pub fn set_opaque(&self, pool: &mut UiObjectPool, id: ScrollBoxId, opaque: bool) -> bool {
    let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
      return false;
    };
    state.options.opaque = opaque;
    true
  }

  /// Return the scroll x in terminal columns for the addressed object when it is available.
  pub fn scroll_x(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<u16> {
    Some(pool.scroll_boxes.boxes.get(&id)?.scroll_x)
  }

  /// Return the scroll y in terminal rows for the addressed object when it is available.
  pub fn scroll_y(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<u16> {
    Some(pool.scroll_boxes.boxes.get(&id)?.scroll_y)
  }

  /// Return the max scroll x in terminal columns for the addressed object when it is available.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn max_scroll_x(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<u16> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    Some(max_scroll_x(state, layout.developer_size()))
  }

  /// Return the max scroll y in terminal rows for the addressed object when it is available.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn max_scroll_y(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<u16> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    Some(max_scroll_y(state, layout.developer_size()))
  }

  /// Return the scroll position for the addressed object when it is available.
  pub fn scroll_position(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<(u16, u16)> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    Some((state.scroll_x, state.scroll_y))
  }

  /// Return the viewport rect for the addressed object when it is available.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn viewport_rect(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<Rect> {
    self.resolved_rect(pool, id, layout)
  }

  /// Return the content width in terminal columns for the addressed object when it is available.
  pub fn content_width(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<u16> {
    Some(pool.scroll_boxes.boxes.get(&id)?.options.content_width)
  }

  /// Return the content height in terminal rows for the addressed object when it is available.
  pub fn content_height(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<u16> {
    Some(pool.scroll_boxes.boxes.get(&id)?.options.content_height)
  }

  /// Return the currently visible rectangle in scrollable content coordinates.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn visible_content_rect(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<Rect> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    let effective = effective_viewport(state, layout.developer_size());
    Some(Rect {
      x: state.scroll_x,
      y: state.scroll_y,
      width: effective
        .width
        .min(state.options.content_width.saturating_sub(state.scroll_x)),
      height: effective
        .height
        .min(state.options.content_height.saturating_sub(state.scroll_y)),
    })
  }

  /// Return the visible content size for the addressed object when it is available.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn visible_content_size(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<Size> {
    let rect = self.visible_content_rect(pool, id, layout)?;
    Some(Size {
      width: rect.width,
      height: rect.height,
    })
  }

  /// Return the visible content width in terminal columns for the addressed object when it is
  /// available.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn visible_content_width(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<u16> {
    Some(self.visible_content_rect(pool, id, layout)?.width)
  }

  /// Return the visible content height in terminal rows for the addressed object when it is
  /// available.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn visible_content_height(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<u16> {
    Some(self.visible_content_rect(pool, id, layout)?.height)
  }

  /// Map a visible content point into developer-viewport coordinates.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `content_x` - The content x.
  /// * `content_y` - The content y.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn content_to_viewport_point(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    content_x: u16,
    content_y: u16,
    layout: &LayoutService,
  ) -> Option<(u16, u16)> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    let viewport_x = content_x.checked_sub(state.scroll_x)?;
    let viewport_y = content_y.checked_sub(state.scroll_y)?;
    let content_viewport =
      resolve_scroll_box_layout(state, layout.developer_size()).content_viewport_rect;
    (viewport_x < content_viewport.width && viewport_y < content_viewport.height)
      .then_some((viewport_x, viewport_y))
  }

  /// Map a developer-viewport point into the scrollable content coordinates when visible.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `viewport_x` - The viewport x.
  /// * `viewport_y` - The viewport y.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn viewport_to_content_point(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    viewport_x: u16,
    viewport_y: u16,
    layout: &LayoutService,
  ) -> Option<(u16, u16)> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    let content_viewport =
      resolve_scroll_box_layout(state, layout.developer_size()).content_viewport_rect;
    if viewport_x >= content_viewport.width || viewport_y >= content_viewport.height {
      return None;
    }
    let content_x = state.scroll_x.saturating_add(viewport_x);
    let content_y = state.scroll_y.saturating_add(viewport_y);
    Some((content_x, content_y))
  }

  /// Drain scroll-position changes queued since the previous read.
  pub fn drain_scroll_events(&self, pool: &mut UiObjectPool) -> Vec<ScrollBoxEvent> {
    pool.scroll_boxes.events.drain(..).collect()
  }

  /// Set absolute content scroll offsets, clamp them to valid ranges, and queue a change event
  /// when needed.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `x` - The horizontal coordinate in terminal cells.
  /// * `y` - The vertical coordinate in terminal cells.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn scroll_to(
    &self,
    pool: &mut UiObjectPool,
    id: ScrollBoxId,
    x: u16,
    y: u16,
    layout: &LayoutService,
  ) -> bool {
    let viewport = layout.developer_size();
    let (old, new, emit) = {
      let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
        return false;
      };
      let max_x = max_scroll_x(state, viewport);
      let max_y = max_scroll_y(state, viewport);
      let old = (state.scroll_x, state.scroll_y);
      state.scroll_x = x.min(max_x);
      state.scroll_y = y.min(max_y);
      let new = (state.scroll_x, state.scroll_y);
      (old, new, state.options.emit_scroll_events)
    };
    if old != new && emit {
      pool
        .scroll_boxes
        .events
        .push_back(ScrollBoxEvent::Scrolled {
          id,
          x: new.0,
          y: new.1,
        });
    }
    true
  }

  /// Apply relative content scroll offsets within valid ranges.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `dx` - The dx.
  /// * `dy` - The dy.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn scroll_by(
    &self,
    pool: &mut UiObjectPool,
    id: ScrollBoxId,
    dx: i32,
    dy: i32,
    layout: &LayoutService,
  ) -> bool {
    let viewport = layout.developer_size();
    let (old, new, emit) = {
      let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
        return false;
      };
      let old = (state.scroll_x, state.scroll_y);
      let nx = (state.scroll_x as i32).saturating_add(dx).max(0) as u16;
      let ny = (state.scroll_y as i32).saturating_add(dy).max(0) as u16;
      state.scroll_x = nx.min(max_scroll_x(state, viewport));
      state.scroll_y = ny.min(max_scroll_y(state, viewport));
      let new = (state.scroll_x, state.scroll_y);
      (old, new, state.options.emit_scroll_events)
    };
    if old != new && emit {
      pool
        .scroll_boxes
        .events
        .push_back(ScrollBoxEvent::Scrolled {
          id,
          x: new.0,
          y: new.1,
        });
    }
    true
  }

  /// Move content to its minimum vertical scroll offset.
  pub fn scroll_to_top(&self, pool: &mut UiObjectPool, id: ScrollBoxId) -> bool {
    let (old, new, emit) = {
      let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
        return false;
      };
      let old = (state.scroll_x, state.scroll_y);
      state.scroll_y = 0;
      let new = (state.scroll_x, state.scroll_y);
      (old, new, state.options.emit_scroll_events)
    };
    if old != new && emit {
      pool
        .scroll_boxes
        .events
        .push_back(ScrollBoxEvent::Scrolled {
          id,
          x: new.0,
          y: new.1,
        });
    }
    true
  }

  /// Move content to its maximum vertical scroll offset.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn scroll_to_bottom(
    &self,
    pool: &mut UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> bool {
    let (old, new, emit) = {
      let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
        return false;
      };
      let old = (state.scroll_x, state.scroll_y);
      state.scroll_y = max_scroll_y(state, layout.developer_size());
      let new = (state.scroll_x, state.scroll_y);
      (old, new, state.options.emit_scroll_events)
    };
    if old != new && emit {
      pool
        .scroll_boxes
        .events
        .push_back(ScrollBoxEvent::Scrolled {
          id,
          x: new.0,
          y: new.1,
        });
    }
    true
  }

  /// Move content to its minimum horizontal scroll offset.
  pub fn scroll_to_left(&self, pool: &mut UiObjectPool, id: ScrollBoxId) -> bool {
    let (old, new, emit) = {
      let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
        return false;
      };
      let old = (state.scroll_x, state.scroll_y);
      state.scroll_x = 0;
      let new = (state.scroll_x, state.scroll_y);
      (old, new, state.options.emit_scroll_events)
    };
    if old != new && emit {
      pool
        .scroll_boxes
        .events
        .push_back(ScrollBoxEvent::Scrolled {
          id,
          x: new.0,
          y: new.1,
        });
    }
    true
  }

  /// Move content to its maximum horizontal scroll offset.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `layout` - The service resolving terminal sizes and positions.
  pub fn scroll_to_right(
    &self,
    pool: &mut UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> bool {
    let (old, new, emit) = {
      let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
        return false;
      };
      let old = (state.scroll_x, state.scroll_y);
      state.scroll_x = max_scroll_x(state, layout.developer_size());
      let new = (state.scroll_x, state.scroll_y);
      (old, new, state.options.emit_scroll_events)
    };
    if old != new && emit {
      pool
        .scroll_boxes
        .events
        .push_back(ScrollBoxEvent::Scrolled {
          id,
          x: new.0,
          y: new.1,
        });
    }
    true
  }

  /// Move the surface to the front of its composition group.
  pub fn bring_to_front(&self, pool: &mut UiObjectPool, id: ScrollBoxId) -> bool {
    pool.move_surface_to_edge(SurfaceId::ScrollBox(id), false)
  }

  /// Move the surface to the back of its composition group.
  pub fn send_to_back(&self, pool: &mut UiObjectPool, id: ScrollBoxId) -> bool {
    pool.move_surface_to_edge(SurfaceId::ScrollBox(id), true)
  }

  /// Place the surface immediately above the referenced peer.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `target` - The object or resource affected by the operation.
  pub fn move_above(&self, pool: &mut UiObjectPool, id: ScrollBoxId, target: SurfaceId) -> bool {
    pool.move_surface_relative(SurfaceId::ScrollBox(id), target, true)
  }

  /// Place the surface immediately below the referenced peer.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `target` - The object or resource affected by the operation.
  pub fn move_below(&self, pool: &mut UiObjectPool, id: ScrollBoxId, target: SurfaceId) -> bool {
    pool.move_surface_relative(SurfaceId::ScrollBox(id), target, false)
  }

  /// Route a pointer event through the component's current hit regions and focus state.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `canvas` - The clipped canvas used for drawing.
  /// * `layout` - The service resolving terminal sizes and positions.
  /// * `event` - The event to apply or route.
  pub fn route_mouse_event(
    &self,
    pool: &mut UiObjectPool,
    canvas: &CanvasService,
    layout: &LayoutService,
    event: MouseEvent,
  ) -> bool {
    // Give an existing drag first refusal on motion and release events before testing new hits.

    if let Some(drag) = pool.scroll_boxes.drag {
      return self.route_drag_or_release(pool, layout, event, drag);
    }

    match event.kind {
      MouseEventKind::Scroll => self.route_wheel(pool, canvas, layout, event),
      MouseEventKind::Press => self.route_press(pool, canvas, layout, event),
      _ => false,
    }
  }

  fn route_wheel(
    &self,
    pool: &mut UiObjectPool,
    canvas: &CanvasService,
    layout: &LayoutService,
    event: MouseEvent,
  ) -> bool {
    let Some(id) = canvas.top_scroll_box_at(event.x, event.y) else {
      return false;
    };
    let Some(state) = pool.scroll_boxes.boxes.get(&id) else {
      return false;
    };
    if !state.options.mouse_wheel {
      return false;
    }
    let v_step = state.options.wheel_step as i32;
    let h_step = state.options.h_wheel_step as i32;

    let (dx, dy) = match event.scroll {
      Some(ScrollDirection::Up) => (0, -v_step),
      Some(ScrollDirection::Down) => (0, v_step),
      Some(ScrollDirection::Left) => (-h_step, 0),
      Some(ScrollDirection::Right) => (h_step, 0),
      _ => return false,
    };

    let effective_dx = if state.options.overflow_x == Overflow::Hidden {
      0
    } else {
      dx
    };
    let effective_dy = if state.options.overflow_y == Overflow::Hidden {
      0
    } else {
      dy
    };

    if effective_dx == 0 && effective_dy == 0 {
      return false;
    }

    self.scroll_by(pool, id, effective_dx, effective_dy, layout)
  }

  fn route_press(
    &self,
    pool: &mut UiObjectPool,
    canvas: &CanvasService,
    layout: &LayoutService,
    event: MouseEvent,
  ) -> bool {
    let Some(button) = event.button else {
      return false;
    };
    let viewport = layout.developer_size();

    let Some(id) = find_scroll_box_for_interaction(pool, canvas, layout, event.x, event.y) else {
      return false;
    };
    let Some(state) = pool.scroll_boxes.boxes.get(&id) else {
      return false;
    };
    let resolved = resolve_scroll_box_layout(state, viewport);

    if let Some(thumb) = resolved.vertical_thumb_rect {
      let physical = scrollbar_physical_rect(thumb, canvas.viewport());
      if physical.contains(event.x, event.y) {
        let bar = resolved.vertical_track_rect.unwrap();

        let thumb_local = thumb.y.saturating_sub(bar.y);
        pool.scroll_boxes.drag = Some(ScrollBoxDragState {
          scroll_box_id: id,
          axis: ScrollbarAxis::Vertical,
          button,
          drag_start_mouse: event.y,
          drag_start_thumb_pos: thumb_local,
          thumb_size: thumb.height,
          track_size: bar.height,
          max_scroll: resolved.max_scroll_y,
        });
        return true;
      }
    }

    if let Some(bar) = resolved.vertical_track_rect {
      let physical = scrollbar_physical_rect(bar, canvas.viewport());
      if physical.contains(event.x, event.y) {
        let step = resolved.content_viewport_rect.height as i32;
        if event.y
          < physical.y.saturating_add(
            resolved
              .vertical_thumb_rect
              .map(|thumb| thumb.y.saturating_sub(bar.y))
              .unwrap_or(0),
          )
        {
          self.scroll_by(pool, id, 0, -step, layout);
        } else {
          self.scroll_by(pool, id, 0, step, layout);
        }
        return true;
      }
    }

    if let Some(thumb) = resolved.horizontal_thumb_rect {
      let physical = scrollbar_physical_rect(thumb, canvas.viewport());
      if physical.contains(event.x, event.y) {
        let bar = resolved.horizontal_track_rect.unwrap();

        let thumb_local = thumb.x.saturating_sub(bar.x);
        pool.scroll_boxes.drag = Some(ScrollBoxDragState {
          scroll_box_id: id,
          axis: ScrollbarAxis::Horizontal,
          button,
          drag_start_mouse: event.x,
          drag_start_thumb_pos: thumb_local,
          thumb_size: thumb.width,
          track_size: bar.width,
          max_scroll: resolved.max_scroll_x,
        });
        return true;
      }
    }

    if let Some(bar) = resolved.horizontal_track_rect {
      let physical = scrollbar_physical_rect(bar, canvas.viewport());
      if physical.contains(event.x, event.y) {
        let step = resolved.content_viewport_rect.width as i32;
        if event.x
          < physical.x.saturating_add(
            resolved
              .horizontal_thumb_rect
              .map(|thumb| thumb.x.saturating_sub(bar.x))
              .unwrap_or(0),
          )
        {
          self.scroll_by(pool, id, -step, 0, layout);
        } else {
          self.scroll_by(pool, id, step, 0, layout);
        }
        return true;
      }
    }

    false
  }

  fn route_drag_or_release(
    &self,
    pool: &mut UiObjectPool,
    layout: &LayoutService,
    event: MouseEvent,
    drag: ScrollBoxDragState,
  ) -> bool {
    match event.kind {
      MouseEventKind::Release => {
        if event.button == Some(drag.button) {
          pool.scroll_boxes.drag = None;
        }
        true
      }
      MouseEventKind::Drag => {
        let mouse_pos = match drag.axis {
          ScrollbarAxis::Vertical => event.y,
          ScrollbarAxis::Horizontal => event.x,
        };
        let new_scroll = drag.scroll_from_mouse(mouse_pos);
        let id = drag.scroll_box_id;
        let (old, new, emit) = {
          let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
            pool.scroll_boxes.drag = None;
            return false;
          };
          let viewport = layout.developer_size();
          let old = (state.scroll_x, state.scroll_y);
          match drag.axis {
            ScrollbarAxis::Vertical => {
              state.scroll_y = new_scroll.min(max_scroll_y(state, viewport));
            }
            ScrollbarAxis::Horizontal => {
              state.scroll_x = new_scroll.min(max_scroll_x(state, viewport));
            }
          }
          let new = (state.scroll_x, state.scroll_y);
          (old, new, state.options.emit_scroll_events)
        };
        if old != new && emit {
          pool
            .scroll_boxes
            .events
            .push_back(ScrollBoxEvent::Scrolled {
              id,
              x: new.0,
              y: new.1,
            });
        }
        // Keep the original press anchors fixed; repeated scroll-to-thumb round trips accumulate
        // integer rounding and make the thumb drift.

        true
      }
      _ => {
        // An active drag owns all pointer events until it ends.

        true
      }
    }
  }
}

fn validate_scrollbar_chars(options: &mut ScrollBoxOptions) {
  let default = ScrollbarStyle::default();
  if char_width(options.scrollbar_style.track_char) != 1 {
    options.scrollbar_style.track_char = default.track_char;
  }
  if char_width(options.scrollbar_style.thumb_char) != 1 {
    options.scrollbar_style.thumb_char = default.thumb_char;
  }
  if char_width(options.scrollbar_style.h_track_char) != 1 {
    options.scrollbar_style.h_track_char = default.h_track_char;
  }
  if char_width(options.scrollbar_style.h_thumb_char) != 1 {
    options.scrollbar_style.h_thumb_char = default.h_thumb_char;
  }
}

fn valid_options(options: &ScrollBoxOptions) -> bool {
  options.wheel_step > 0
}

/// Clamp a configured rectangle to its supported coordinate and size limits.
pub(crate) fn clamp_rect(rect: Rect, viewport: Size) -> Rect {
  let x = rect.x.min(viewport.width);
  let y = rect.y.min(viewport.height);
  Rect {
    x,
    y,
    width: rect.width.min(viewport.width.saturating_sub(x)),
    height: rect.height.min(viewport.height.saturating_sub(y)),
  }
}

/// Resolve content bounds and scrollbar occupancy until the viewport dimensions are consistent.
pub(crate) fn resolve_scroll_box_layout(
  state: &ScrollBoxState,
  viewport: Size,
) -> ResolvedScrollBoxLayout {
  let viewport_rect = clamp_rect(state.options.rect, viewport);
  let (vertical, horizontal) = resolved_scrollbar_visibility(state, viewport_rect);
  let content_viewport_rect = Rect {
    x: viewport_rect.x,
    y: viewport_rect.y,
    width: viewport_rect.width.saturating_sub(u16::from(vertical)),
    height: viewport_rect.height.saturating_sub(u16::from(horizontal)),
  };

  let vertical_track_rect = vertical
    .then(|| {
      let x = match state.options.scrollbar_layout {
        ScrollbarLayout::Overlay | ScrollbarLayout::Inside => viewport_rect
          .x
          .saturating_add(viewport_rect.width)
          .saturating_sub(1),
        ScrollbarLayout::ReserveSpace => {
          let outside_x = viewport_rect.x.saturating_add(viewport_rect.width);
          if outside_x < viewport.width {
            outside_x
          } else {
            outside_x.saturating_sub(1)
          }
        }
      };
      Rect {
        x,
        y: viewport_rect.y,
        width: 1,
        height: content_viewport_rect.height,
      }
    })
    .filter(|rect| rect.width > 0 && rect.height > 0 && rect.x < viewport.width);
  let horizontal_track_rect = horizontal
    .then(|| {
      let y = match state.options.scrollbar_layout {
        ScrollbarLayout::Overlay | ScrollbarLayout::Inside => viewport_rect
          .y
          .saturating_add(viewport_rect.height)
          .saturating_sub(1),
        ScrollbarLayout::ReserveSpace => {
          let outside_y = viewport_rect.y.saturating_add(viewport_rect.height);
          if outside_y < viewport.height {
            outside_y
          } else {
            outside_y.saturating_sub(1)
          }
        }
      };
      Rect {
        x: viewport_rect.x,
        y,
        width: content_viewport_rect.width,
        height: 1,
      }
    })
    .filter(|rect| rect.width > 0 && rect.height > 0 && rect.y < viewport.height);

  let max_scroll_x = if content_viewport_rect.width > 0 {
    state
      .options
      .content_width
      .saturating_sub(content_viewport_rect.width)
  } else {
    0
  };
  let max_scroll_y = if content_viewport_rect.height > 0 {
    state
      .options
      .content_height
      .saturating_sub(content_viewport_rect.height)
  } else {
    0
  };
  let vertical_thumb_rect = vertical_track_rect.map(|track| {
    scrollbar_thumb_rect(
      track,
      ScrollbarAxis::Vertical,
      state.scroll_y,
      max_scroll_y,
      state.options.content_height,
      state.options.scrollbar_style.minimum_thumb_height,
    )
  });
  let horizontal_thumb_rect = horizontal_track_rect.map(|track| {
    scrollbar_thumb_rect(
      track,
      ScrollbarAxis::Horizontal,
      state.scroll_x,
      max_scroll_x,
      state.options.content_width,
      state.options.scrollbar_style.minimum_thumb_height,
    )
  });
  let occupied_rect = [vertical_track_rect, horizontal_track_rect]
    .into_iter()
    .flatten()
    .fold(viewport_rect, union_rect);

  ResolvedScrollBoxLayout {
    viewport_rect,
    content_viewport_rect,
    occupied_rect,
    vertical_track_rect,
    horizontal_track_rect,
    vertical_thumb_rect,
    horizontal_thumb_rect,
    max_scroll_x,
    max_scroll_y,
  }
}

fn resolved_scrollbar_visibility(state: &ScrollBoxState, rect: Rect) -> (bool, bool) {
  let vertical_policy = state.options.scrollbar.vertical;
  let horizontal_policy = state.options.scrollbar.horizontal;
  let mut vertical = scrollbar_visible(
    vertical_policy,
    state.options.content_height,
    rect.height,
    rect.height,
  );
  let mut horizontal = scrollbar_visible(
    horizontal_policy,
    state.options.content_width,
    rect.width,
    rect.width,
  );

  loop {
    let next_vertical = scrollbar_visible(
      vertical_policy,
      state.options.content_height,
      rect.height.saturating_sub(u16::from(horizontal)),
      rect.height,
    );
    let next_horizontal = scrollbar_visible(
      horizontal_policy,
      state.options.content_width,
      rect.width.saturating_sub(u16::from(vertical)),
      rect.width,
    );
    if next_vertical == vertical && next_horizontal == horizontal {
      return (vertical, horizontal);
    }
    vertical = next_vertical;
    horizontal = next_horizontal;
  }
}

fn scrollbar_visible(
  policy: ScrollbarVisibility,
  content: u16,
  available: u16,
  axis_extent: u16,
) -> bool {
  match policy {
    ScrollbarVisibility::Always => axis_extent > 0,
    ScrollbarVisibility::Auto => axis_extent > 0 && content > available,
    ScrollbarVisibility::Never => false,
  }
}

fn scrollbar_thumb_rect(
  track: Rect,
  axis: ScrollbarAxis,
  scroll: u16,
  max_scroll: u16,
  content_extent: u16,
  minimum_thumb_extent: u16,
) -> Rect {
  let track_extent = match axis {
    ScrollbarAxis::Vertical => track.height,
    ScrollbarAxis::Horizontal => track.width,
  };
  let thumb_extent = if max_scroll == 0 {
    track_extent
  } else {
    ((u32::from(track_extent) * u32::from(track_extent)) / u32::from(content_extent.max(1)))
      .max(u32::from(minimum_thumb_extent))
      .min(u32::from(track_extent)) as u16
  };
  let travel = track_extent.saturating_sub(thumb_extent);
  let thumb_offset = if max_scroll == 0 {
    0
  } else {
    (u32::from(scroll.min(max_scroll)) * u32::from(travel) / u32::from(max_scroll)) as u16
  };
  match axis {
    ScrollbarAxis::Vertical => Rect {
      x: track.x,
      y: track.y.saturating_add(thumb_offset),
      width: track.width,
      height: thumb_extent,
    },
    ScrollbarAxis::Horizontal => Rect {
      x: track.x.saturating_add(thumb_offset),
      y: track.y,
      width: thumb_extent,
      height: track.height,
    },
  }
}

fn union_rect(left: Rect, right: Rect) -> Rect {
  let x = left.x.min(right.x);
  let y = left.y.min(right.y);
  let right_edge = left
    .x
    .saturating_add(left.width)
    .max(right.x.saturating_add(right.width));
  let bottom_edge = left
    .y
    .saturating_add(left.height)
    .max(right.y.saturating_add(right.height));
  Rect {
    x,
    y,
    width: right_edge.saturating_sub(x),
    height: bottom_edge.saturating_sub(y),
  }
}

/// Return the visible content size after scrollbar occupancy is resolved.
pub(crate) fn effective_viewport(state: &ScrollBoxState, viewport: Size) -> Size {
  let rect = resolve_scroll_box_layout(state, viewport).content_viewport_rect;
  Size {
    width: rect.width,
    height: rect.height,
  }
}

/// Report whether vertical overflow requires a visible scrollbar.
#[cfg(test)]
pub(crate) fn shows_vertical_scrollbar(state: &ScrollBoxState, viewport: Size) -> bool {
  resolve_scroll_box_layout(state, viewport)
    .vertical_track_rect
    .is_some()
}

/// Report whether horizontal overflow requires a visible scrollbar.
#[cfg(test)]
pub(crate) fn shows_horizontal_scrollbar(state: &ScrollBoxState, viewport: Size) -> bool {
  resolve_scroll_box_layout(state, viewport)
    .horizontal_track_rect
    .is_some()
}

/// Return the max scroll x in terminal columns for the addressed object.
pub(crate) fn max_scroll_x(state: &ScrollBoxState, viewport: Size) -> u16 {
  resolve_scroll_box_layout(state, viewport).max_scroll_x
}

/// Return the max scroll y in terminal rows for the addressed object.
pub(crate) fn max_scroll_y(state: &ScrollBoxState, viewport: Size) -> u16 {
  resolve_scroll_box_layout(state, viewport).max_scroll_y
}

/// Clamp the stored scroll offsets to the currently valid content range.
pub(crate) fn clamp_scroll(state: &mut ScrollBoxState, viewport: Size) {
  state.scroll_x = state.scroll_x.min(max_scroll_x(state, viewport));
  state.scroll_y = state.scroll_y.min(max_scroll_y(state, viewport));
}

fn scrollbar_physical_rect(rect: Rect, viewport: Rect) -> Rect {
  Rect {
    x: viewport.x.saturating_add(rect.x),
    y: viewport.y.saturating_add(rect.y),
    width: rect.width,
    height: rect.height,
  }
}

fn find_scroll_box_for_interaction(
  pool: &UiObjectPool,
  canvas: &CanvasService,
  layout: &LayoutService,
  x: u16,
  y: u16,
) -> Option<ScrollBoxId> {
  let viewport_size = layout.developer_size();
  canvas
    .surface_order()
    .iter()
    .rev()
    .filter_map(|surface| match surface {
      SurfaceId::ScrollBox(id) => {
        let state = pool.scroll_boxes.boxes.get(id)?;
        if !state.options.visible {
          return None;
        }
        let occupied = resolve_scroll_box_layout(state, viewport_size).occupied_rect;
        scrollbar_physical_rect(occupied, canvas.viewport())
          .contains(x, y)
          .then_some(*id)
      }
      _ => None,
    })
    .next()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{SliceOptions, SliceService};
  use tg_core_input::{MouseButton, ScrollDirection};
  use tg_service_canvas::CanvasService;

  #[test]
  fn create_rejects_zero_wheel_step_and_allows_horizontal_overflow() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();

    assert!(
      service
        .create(
          &mut pool,
          ScrollBoxOptions {
            overflow_x: Overflow::Auto,
            ..Default::default()
          }
        )
        .is_some()
    );

    assert!(
      service
        .create(
          &mut pool,
          ScrollBoxOptions {
            wheel_step: 0,
            ..Default::default()
          }
        )
        .is_none()
    );
  }

  #[test]
  fn scroll_is_clamped_to_content() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 4,
          },
          content_width: 20,
          content_height: 10,
          overflow_x: Overflow::Auto,
          scrollbar_layout: ScrollbarLayout::Overlay,
          ..Default::default()
        },
      )
      .unwrap();

    assert!(service.scroll_to(&mut pool, id, 0, 99, &layout));
    assert_eq!(service.scroll_y(&pool, id), Some(6));
    assert!(service.scroll_by(&mut pool, id, 0, -10, &layout));
    assert_eq!(service.scroll_y(&pool, id), Some(0));
    assert!(service.scroll_to_bottom(&mut pool, id, &layout));
    assert_eq!(service.scroll_y(&pool, id), Some(6));

    assert!(service.scroll_to(&mut pool, id, 99, 0, &layout));
    assert_eq!(service.scroll_x(&pool, id), Some(13));
    assert!(service.scroll_by(&mut pool, id, -20, 0, &layout));
    assert_eq!(service.scroll_x(&pool, id), Some(0));
    assert!(service.scroll_to_right(&mut pool, id, &layout));
    assert_eq!(service.scroll_x(&pool, id), Some(13));
    assert!(service.scroll_to_left(&mut pool, id));
    assert_eq!(service.scroll_x(&pool, id), Some(0));
  }

  #[test]
  fn scroll_position_and_query_api() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 1,
            y: 2,
            width: 8,
            height: 4,
          },
          content_width: 16,
          content_height: 12,
          overflow_x: Overflow::Auto,
          scrollbar_layout: ScrollbarLayout::Overlay,
          ..Default::default()
        },
      )
      .unwrap();

    assert_eq!(service.scroll_position(&pool, id), Some((0, 0)));
    assert_eq!(service.content_width(&pool, id), Some(16));
    assert_eq!(service.content_height(&pool, id), Some(12));
    assert!(service.viewport_rect(&pool, id, &layout).is_some());
    let visible = service.visible_content_rect(&pool, id, &layout).unwrap();
    assert_eq!(visible.x, 0);
    assert_eq!(visible.y, 0);
    assert_eq!(visible.width, 7);
    assert_eq!(visible.height, 4);

    service.scroll_to(&mut pool, id, 3, 5, &layout);
    let visible = service.visible_content_rect(&pool, id, &layout).unwrap();
    assert_eq!(visible.x, 3);
    assert_eq!(visible.y, 5);
  }

  #[test]
  fn coordinate_conversion() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 4,
          },
          content_width: 16,
          content_height: 12,
          overflow_x: Overflow::Auto,
          ..Default::default()
        },
      )
      .unwrap();

    service.scroll_to(&mut pool, id, 3, 5, &layout);

    assert_eq!(
      service.content_to_viewport_point(&pool, id, 3, 5, &layout),
      Some((0, 0))
    );
    assert_eq!(
      service.content_to_viewport_point(&pool, id, 0, 0, &layout),
      None
    );

    assert_eq!(
      service.viewport_to_content_point(&pool, id, 0, 0, &layout),
      Some((3, 5))
    );
    assert_eq!(
      service.viewport_to_content_point(&pool, id, 1, 1, &layout),
      Some((4, 6))
    );
  }

  #[test]
  fn scroll_events_emitted_when_enabled() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 4,
          },
          content_width: 8,
          content_height: 10,
          emit_scroll_events: true,
          ..Default::default()
        },
      )
      .unwrap();

    service.scroll_by(&mut pool, id, 0, 3, &layout);
    let events = service.drain_scroll_events(&mut pool);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], ScrollBoxEvent::Scrolled { id, x: 0, y: 3 });

    service.scroll_to(&mut pool, id, 0, 3, &layout);
    let events = service.drain_scroll_events(&mut pool);
    assert!(events.is_empty());
  }

  #[test]
  fn scroll_events_not_emitted_when_disabled() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 4,
          },
          content_width: 8,
          content_height: 10,
          emit_scroll_events: false,
          ..Default::default()
        },
      )
      .unwrap();

    service.scroll_by(&mut pool, id, 0, 3, &layout);
    let events = service.drain_scroll_events(&mut pool);
    assert!(events.is_empty());
  }

  #[test]
  fn mouse_wheel_scrolls_only_when_scroll_box_is_top_surface() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 4,
          },
          content_width: 8,
          content_height: 10,
          wheel_step: 2,
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    pool.prepare_canvas(&mut canvas, &layout);

    assert!(service.route_mouse_event(
      &mut pool,
      &canvas,
      &layout,
      MouseEvent {
        kind: MouseEventKind::Scroll,
        button: None,
        scroll: Some(ScrollDirection::Down),
        x: 1,
        y: 1,
      }
    ));
    assert_eq!(service.scroll_y(&pool, id), Some(2));

    let slice = SliceService::new()
      .create(&mut pool, SliceOptions::default())
      .unwrap();
    pool.move_surface_relative(SurfaceId::Slice(slice), SurfaceId::ScrollBox(id), true);
    canvas.begin_frame(&layout);
    pool.prepare_canvas(&mut canvas, &layout);
    assert!(!service.route_mouse_event(
      &mut pool,
      &canvas,
      &layout,
      MouseEvent {
        kind: MouseEventKind::Scroll,
        button: None,
        scroll: Some(ScrollDirection::Down),
        x: 1,
        y: 1,
      }
    ));
    assert_eq!(service.scroll_y(&pool, id), Some(2));
  }

  #[test]
  fn horizontal_wheel_scrolls_when_overflow_x_auto() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 4,
          },
          content_width: 16,
          content_height: 4,
          overflow_x: Overflow::Auto,
          wheel_step: 2,
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    pool.prepare_canvas(&mut canvas, &layout);

    assert!(service.route_mouse_event(
      &mut pool,
      &canvas,
      &layout,
      MouseEvent {
        kind: MouseEventKind::Scroll,
        button: None,
        scroll: Some(ScrollDirection::Right),
        x: 1,
        y: 1,
      }
    ));
    assert_eq!(service.scroll_x(&pool, id), Some(2));
  }

  #[test]
  fn scrollbar_drag_starts_and_ends() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 15);
    let _id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 4,
          },
          content_width: 8,
          content_height: 12,
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    pool.prepare_canvas(&mut canvas, &layout);

    let pressed = service.route_mouse_event(
      &mut pool,
      &canvas,
      &layout,
      MouseEvent {
        kind: MouseEventKind::Press,
        button: Some(MouseButton::Left),
        scroll: None,
        x: 7,
        y: 0,
      },
    );
    assert!(pressed, "press on thumb should be consumed");
    assert!(pool.scroll_boxes.drag.is_some());

    let released = service.route_mouse_event(
      &mut pool,
      &canvas,
      &layout,
      MouseEvent {
        kind: MouseEventKind::Release,
        button: Some(MouseButton::Left),
        scroll: None,
        x: 7,
        y: 3,
      },
    );
    assert!(released);
    assert!(pool.scroll_boxes.drag.is_none());
  }

  #[test]
  fn scrollbar_track_click_pages() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 15);
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 4,
          },
          content_width: 8,
          content_height: 12,
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    pool.prepare_canvas(&mut canvas, &layout);

    service.scroll_to_top(&mut pool, id);
    let pressed = service.route_mouse_event(
      &mut pool,
      &canvas,
      &layout,
      MouseEvent {
        kind: MouseEventKind::Press,
        button: Some(MouseButton::Left),
        scroll: None,
        x: 7,
        y: 3,
      },
    );
    assert!(pressed);

    assert!(service.scroll_y(&pool, id).unwrap() > 0);
  }

  #[test]
  fn overflow_hidden_blocks_wheel() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 4,
          },
          content_width: 16,
          content_height: 4,
          overflow_x: Overflow::Hidden,
          overflow_y: Overflow::Hidden,
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    pool.prepare_canvas(&mut canvas, &layout);

    assert!(!service.route_mouse_event(
      &mut pool,
      &canvas,
      &layout,
      MouseEvent {
        kind: MouseEventKind::Scroll,
        button: None,
        scroll: Some(ScrollDirection::Down),
        x: 1,
        y: 1,
      }
    ));
    assert!(!service.route_mouse_event(
      &mut pool,
      &canvas,
      &layout,
      MouseEvent {
        kind: MouseEventKind::Scroll,
        button: None,
        scroll: Some(ScrollDirection::Right),
        x: 1,
        y: 1,
      }
    ));
    assert_eq!(service.scroll_x(&pool, id), Some(0));
    assert_eq!(service.scroll_y(&pool, id), Some(0));
  }

  #[test]
  fn effective_viewport_reduces_in_reserve_space() {
    let state = ScrollBoxState {
      options: ScrollBoxOptions {
        rect: Rect {
          x: 0,
          y: 0,
          width: 10,
          height: 5,
        },
        content_width: 20,
        content_height: 10,
        scrollbar_layout: ScrollbarLayout::ReserveSpace,
        scrollbar: ScrollbarPolicy {
          vertical: ScrollbarVisibility::Auto,
          horizontal: ScrollbarVisibility::Auto,
        },
        ..Default::default()
      },
      scroll_x: 0,
      scroll_y: 0,
    };
    let viewport = Size {
      width: 20,
      height: 15,
    };
    // Reserved vertical and horizontal scrollbars each remove one cell from the opposite content
    // dimension.

    let eff = effective_viewport(&state, viewport);
    assert_eq!(eff.width, 9);
    assert_eq!(eff.height, 4);
  }

  #[test]
  fn vertical_auto_scrollbar_can_require_horizontal_scrollbar() {
    let state = ScrollBoxState {
      options: ScrollBoxOptions {
        rect: Rect {
          x: 0,
          y: 0,
          width: 10,
          height: 5,
        },
        content_width: 10,
        content_height: 6,
        scrollbar_layout: ScrollbarLayout::Inside,
        scrollbar: ScrollbarPolicy {
          vertical: ScrollbarVisibility::Auto,
          horizontal: ScrollbarVisibility::Auto,
        },
        ..Default::default()
      },
      scroll_x: 0,
      scroll_y: 0,
    };
    let viewport = Size {
      width: 10,
      height: 5,
    };

    let resolved = resolve_scroll_box_layout(&state, viewport);
    assert!(resolved.vertical_track_rect.is_some());
    assert!(resolved.horizontal_track_rect.is_some());
    assert_eq!(
      resolved.content_viewport_rect,
      Rect {
        x: 0,
        y: 0,
        width: 9,
        height: 4
      }
    );
    assert_eq!(
      resolved.vertical_track_rect,
      Some(Rect {
        x: 9,
        y: 0,
        width: 1,
        height: 4
      })
    );
    assert_eq!(
      resolved.horizontal_track_rect,
      Some(Rect {
        x: 0,
        y: 4,
        width: 9,
        height: 1
      })
    );
    assert_eq!(
      effective_viewport(&state, viewport),
      Size {
        width: 9,
        height: 4
      }
    );
    assert_eq!(max_scroll_x(&state, viewport), 1);
  }

  #[test]
  fn horizontal_auto_scrollbar_can_require_vertical_scrollbar() {
    let state = ScrollBoxState {
      options: ScrollBoxOptions {
        rect: Rect {
          x: 0,
          y: 0,
          width: 10,
          height: 5,
        },
        content_width: 11,
        content_height: 5,
        scrollbar_layout: ScrollbarLayout::Inside,
        scrollbar: ScrollbarPolicy {
          vertical: ScrollbarVisibility::Auto,
          horizontal: ScrollbarVisibility::Auto,
        },
        ..Default::default()
      },
      scroll_x: 0,
      scroll_y: 0,
    };
    let viewport = Size {
      width: 10,
      height: 5,
    };

    assert!(shows_vertical_scrollbar(&state, viewport));
    assert!(shows_horizontal_scrollbar(&state, viewport));
    assert_eq!(
      effective_viewport(&state, viewport),
      Size {
        width: 9,
        height: 4
      }
    );
    assert_eq!(max_scroll_y(&state, viewport), 1);
  }

  #[test]
  fn overlay_scrollbars_use_the_uncovered_visible_area() {
    let state = ScrollBoxState {
      options: ScrollBoxOptions {
        rect: Rect {
          x: 0,
          y: 0,
          width: 10,
          height: 5,
        },
        content_width: 10,
        content_height: 6,
        scrollbar_layout: ScrollbarLayout::Overlay,
        scrollbar: ScrollbarPolicy {
          vertical: ScrollbarVisibility::Auto,
          horizontal: ScrollbarVisibility::Auto,
        },
        ..Default::default()
      },
      scroll_x: 0,
      scroll_y: 0,
    };
    let viewport = Size {
      width: 10,
      height: 5,
    };

    assert!(shows_vertical_scrollbar(&state, viewport));
    assert!(shows_horizontal_scrollbar(&state, viewport));
    assert_eq!(
      effective_viewport(&state, viewport),
      Size {
        width: 9,
        height: 4
      }
    );
  }

  #[test]
  fn viewport_and_visible_content_queries_use_distinct_sizes() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 15);
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 5,
          },
          content_width: 10,
          content_height: 10,
          scrollbar_layout: ScrollbarLayout::Inside,
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Auto,
            horizontal: ScrollbarVisibility::Never,
          },
          ..Default::default()
        },
      )
      .unwrap();

    assert_eq!(
      service.viewport_size(&pool, id, &layout),
      Some(Size {
        width: 10,
        height: 5
      })
    );
    assert_eq!(service.viewport_width(&pool, id, &layout), Some(10));
    assert_eq!(service.viewport_height(&pool, id, &layout), Some(5));
    assert_eq!(
      service.effective_viewport_size(&pool, id, &layout),
      Some(Size {
        width: 9,
        height: 5
      })
    );
    assert_eq!(
      service.visible_content_size(&pool, id, &layout),
      Some(Size {
        width: 9,
        height: 5
      })
    );
    assert_eq!(
      service.content_size(&pool, id),
      Some(Size {
        width: 10,
        height: 10
      })
    );
  }

  #[test]
  fn overlay_scrollbar_reduces_visible_content_queries() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 15);
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 5,
          },
          content_width: 10,
          content_height: 10,
          scrollbar_layout: ScrollbarLayout::Overlay,
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Auto,
            horizontal: ScrollbarVisibility::Never,
          },
          ..Default::default()
        },
      )
      .unwrap();

    assert_eq!(
      service.visible_content_size(&pool, id, &layout),
      Some(Size {
        width: 9,
        height: 5
      })
    );
  }

  #[test]
  fn scrollbar_cells_are_not_content_coordinates() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(10, 5);
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 5,
          },
          content_width: 10,
          content_height: 6,
          overflow_x: Overflow::Auto,
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Auto,
            horizontal: ScrollbarVisibility::Auto,
          },
          ..Default::default()
        },
      )
      .unwrap();

    assert_eq!(
      service.viewport_to_content_point(&pool, id, 8, 3, &layout),
      Some((8, 3))
    );
    assert_eq!(
      service.content_to_viewport_point(&pool, id, 8, 3, &layout),
      Some((8, 3))
    );
    assert_eq!(
      service.viewport_to_content_point(&pool, id, 9, 0, &layout),
      None
    );
    assert_eq!(
      service.viewport_to_content_point(&pool, id, 0, 4, &layout),
      None
    );
    assert_eq!(
      service.content_to_viewport_point(&pool, id, 9, 0, &layout),
      None
    );
    assert_eq!(
      service.content_to_viewport_point(&pool, id, 0, 4, &layout),
      None
    );
  }

  #[test]
  fn reserve_space_scrollbars_remain_visible_at_viewport_edge() {
    let state = ScrollBoxState {
      options: ScrollBoxOptions {
        rect: Rect {
          x: 0,
          y: 0,
          width: 10,
          height: 5,
        },
        content_width: 20,
        content_height: 10,
        scrollbar_layout: ScrollbarLayout::ReserveSpace,
        scrollbar: ScrollbarPolicy {
          vertical: ScrollbarVisibility::Auto,
          horizontal: ScrollbarVisibility::Auto,
        },
        ..Default::default()
      },
      scroll_x: 0,
      scroll_y: 0,
    };
    let resolved = resolve_scroll_box_layout(
      &state,
      Size {
        width: 10,
        height: 5,
      },
    );

    assert_eq!(resolved.vertical_track_rect.unwrap().x, 9);
    assert_eq!(resolved.horizontal_track_rect.unwrap().y, 4);
    assert_eq!(
      resolved.content_viewport_rect,
      Rect {
        x: 0,
        y: 0,
        width: 9,
        height: 4
      }
    );
    assert_eq!(
      resolved.occupied_rect,
      Rect {
        x: 0,
        y: 0,
        width: 10,
        height: 5
      }
    );
  }

  #[test]
  fn max_scroll_x_uses_effective_viewport() {
    let state = ScrollBoxState {
      options: ScrollBoxOptions {
        rect: Rect {
          x: 0,
          y: 0,
          width: 10,
          height: 5,
        },
        content_width: 20,
        content_height: 5,
        scrollbar_layout: ScrollbarLayout::ReserveSpace,
        scrollbar: ScrollbarPolicy {
          vertical: ScrollbarVisibility::Auto,
          horizontal: ScrollbarVisibility::Auto,
        },
        ..Default::default()
      },
      scroll_x: 0,
      scroll_y: 0,
    };
    let viewport = Size {
      width: 20,
      height: 15,
    };
    // One scrollbar can reduce the viewport enough to require the other; resolve both dimensions
    // together.

    assert!(shows_horizontal_scrollbar(&state, viewport));
    assert!(shows_vertical_scrollbar(&state, viewport));
    assert_eq!(max_scroll_x(&state, viewport), 11);
  }

  #[test]
  fn inside_layout_reduces_viewport() {
    let state = ScrollBoxState {
      options: ScrollBoxOptions {
        rect: Rect {
          x: 0,
          y: 0,
          width: 10,
          height: 5,
        },
        content_width: 10,
        content_height: 10,
        scrollbar_layout: ScrollbarLayout::Inside,
        scrollbar: ScrollbarPolicy {
          vertical: ScrollbarVisibility::Auto,
          horizontal: ScrollbarVisibility::Never,
        },
        ..Default::default()
      },
      scroll_x: 0,
      scroll_y: 0,
    };
    let viewport = Size {
      width: 20,
      height: 15,
    };

    let eff = effective_viewport(&state, viewport);
    assert_eq!(eff.width, 9);
    assert_eq!(eff.height, 5);
    assert_eq!(max_scroll_y(&state, viewport), 5);
  }

  #[test]
  fn scrollbar_chars_fall_back_to_default_when_invalid_width() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 4,
          },
          content_width: 8,
          content_height: 1,
          scrollbar_style: ScrollbarStyle {
            track_char: '中', // Reject wide or zero-width scrollbar glyphs; valid scrollbar pieces must occupy
            // exactly one cell.
            thumb_char: '\u{200D}',
            h_track_char: '━',
            h_thumb_char: '\u{200D}',
            ..Default::default()
          },
          ..Default::default()
        },
      )
      .unwrap();
    let state = pool.scroll_boxes.boxes.get(&id).unwrap();
    let style = &state.options.scrollbar_style;
    assert_eq!(style.track_char, '│');
    assert_eq!(style.thumb_char, '█');
    assert_eq!(style.h_track_char, '━');
    assert_eq!(style.h_thumb_char, '█');
  }
}
