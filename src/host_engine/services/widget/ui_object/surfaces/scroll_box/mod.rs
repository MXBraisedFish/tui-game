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
use crate::host_engine::services::ui::UiObjectPool;
use crate::host_engine::services::unicode::char_width;
use crate::host_engine::services::{
  CanvasService, LayoutService, MouseEvent, MouseEventKind, Rect, ScrollDirection, Size,
};

/// 可滚动绘制面服务。
pub struct ScrollBoxService;

impl ScrollBoxService {
  pub fn new() -> Self {
    Self
  }

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

  pub fn remove(&self, pool: &mut UiObjectPool, id: ScrollBoxId) -> bool {
    if pool.scroll_boxes.boxes.remove(&id).is_none() {
      return false;
    }
    pool
      .surfaces
      .retain(|surface| *surface != SurfaceId::ScrollBox(id));
    true
  }

  pub fn exists(&self, pool: &UiObjectPool, id: ScrollBoxId) -> bool {
    pool.scroll_boxes.boxes.contains_key(&id)
  }

  pub fn rect(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<Rect> {
    Some(pool.scroll_boxes.boxes.get(&id)?.options.rect)
  }

  pub fn resolved_rect(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<Rect> {
    let rect = self.rect(pool, id)?;
    Some(clamp_rect(rect, layout.developer_size()))
  }

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

  pub fn viewport_width(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<u16> {
    Some(self.viewport_size(pool, id, layout)?.width)
  }

  pub fn viewport_height(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<u16> {
    Some(self.viewport_size(pool, id, layout)?.height)
  }

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

  pub fn content_size(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<Size> {
    let options = &pool.scroll_boxes.boxes.get(&id)?.options;
    Some(Size {
      width: options.content_width,
      height: options.content_height,
    })
  }

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

  pub fn is_visible(&self, pool: &UiObjectPool, id: ScrollBoxId) -> bool {
    pool
      .scroll_boxes
      .boxes
      .get(&id)
      .is_some_and(|state| state.options.visible)
  }

  pub fn set_visible(&self, pool: &mut UiObjectPool, id: ScrollBoxId, visible: bool) -> bool {
    let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
      return false;
    };
    state.options.visible = visible;
    true
  }

  pub fn is_opaque(&self, pool: &UiObjectPool, id: ScrollBoxId) -> bool {
    pool
      .scroll_boxes
      .boxes
      .get(&id)
      .is_some_and(|state| state.options.opaque)
  }

  pub fn set_opaque(&self, pool: &mut UiObjectPool, id: ScrollBoxId, opaque: bool) -> bool {
    let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
      return false;
    };
    state.options.opaque = opaque;
    true
  }

  pub fn scroll_x(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<u16> {
    Some(pool.scroll_boxes.boxes.get(&id)?.scroll_x)
  }

  pub fn scroll_y(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<u16> {
    Some(pool.scroll_boxes.boxes.get(&id)?.scroll_y)
  }

  pub fn max_scroll_x(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<u16> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    Some(max_scroll_x(state, layout.developer_size()))
  }

  pub fn max_scroll_y(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<u16> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    Some(max_scroll_y(state, layout.developer_size()))
  }

  /// 查询完整的滚动位置。
  pub fn scroll_position(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<(u16, u16)> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    Some((state.scroll_x, state.scroll_y))
  }

  /// 查询 viewport 矩形（在 Developer Viewport 内的位置和大小）。
  pub fn viewport_rect(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<Rect> {
    self.resolved_rect(pool, id, layout)
  }

  /// 查询内容区宽度。
  pub fn content_width(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<u16> {
    Some(pool.scroll_boxes.boxes.get(&id)?.options.content_width)
  }

  /// 查询内容区高度。
  pub fn content_height(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<u16> {
    Some(pool.scroll_boxes.boxes.get(&id)?.options.content_height)
  }

  /// 查询当前被滚动窗口看到的内容区域（内容坐标系）。
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

  /// 查询当前可见内容区域的宽度。
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

  pub fn visible_content_width(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<u16> {
    Some(self.visible_content_rect(pool, id, layout)?.width)
  }

  /// 查询当前可见内容区域的高度。
  pub fn visible_content_height(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> Option<u16> {
    Some(self.visible_content_rect(pool, id, layout)?.height)
  }

  /// 将内容坐标转换为视口坐标。
  pub fn content_to_viewport_point(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    content_x: u16,
    content_y: u16,
  ) -> Option<(u16, u16)> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    let viewport_x = content_x.checked_sub(state.scroll_x)?;
    let viewport_y = content_y.checked_sub(state.scroll_y)?;
    Some((viewport_x, viewport_y))
  }

  /// 将视口坐标转换为内容坐标。
  pub fn viewport_to_content_point(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    viewport_x: u16,
    viewport_y: u16,
  ) -> Option<(u16, u16)> {
    let state = pool.scroll_boxes.boxes.get(&id)?;
    let content_x = state.scroll_x.saturating_add(viewport_x);
    let content_y = state.scroll_y.saturating_add(viewport_y);
    Some((content_x, content_y))
  }

  /// 取出所有已排队的滚动事件。
  pub fn drain_scroll_events(&self, pool: &mut UiObjectPool) -> Vec<ScrollBoxEvent> {
    pool.scroll_boxes.events.drain(..).collect()
  }

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

  pub fn bring_to_front(&self, pool: &mut UiObjectPool, id: ScrollBoxId) -> bool {
    pool.move_surface_to_edge(SurfaceId::ScrollBox(id), false)
  }

  pub fn send_to_back(&self, pool: &mut UiObjectPool, id: ScrollBoxId) -> bool {
    pool.move_surface_to_edge(SurfaceId::ScrollBox(id), true)
  }

  pub fn move_above(&self, pool: &mut UiObjectPool, id: ScrollBoxId, target: SurfaceId) -> bool {
    pool.move_surface_relative(SurfaceId::ScrollBox(id), target, true)
  }

  pub fn move_below(&self, pool: &mut UiObjectPool, id: ScrollBoxId, target: SurfaceId) -> bool {
    pool.move_surface_relative(SurfaceId::ScrollBox(id), target, false)
  }

  // ─── 内部事件路由 ────────────────────────────────────

  pub(crate) fn route_mouse_event(
    &self,
    pool: &mut UiObjectPool,
    canvas: &CanvasService,
    layout: &LayoutService,
    event: MouseEvent,
  ) -> bool {
    // 如果有活跃的拖动且鼠标按钮释放或拖拽中，优先处理。
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

    // 根据 overflow 设置限制滚动方向。
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

    // 找到鼠标下方最顶层的 ScrollBox（含滚动条区域）。
    let Some(id) = find_scroll_box_for_interaction(pool, canvas, layout, event.x, event.y) else {
      return false;
    };
    let Some(state) = pool.scroll_boxes.boxes.get(&id) else {
      return false;
    };
    let clamped = clamp_rect(state.options.rect, viewport);

    // 命中测试垂直滚动条滑块。
    if let Some(thumb) = vertical_thumb_rect(state, viewport) {
      let physical = scrollbar_physical_rect(thumb, canvas.viewport());
      if physical.contains(event.x, event.y) {
        let bar = vertical_scrollbar_rect(state, viewport).unwrap();
        // 滑块在轨道内的本地偏移（开发者坐标）。
        let thumb_local = thumb.y.saturating_sub(bar.y);
        pool.scroll_boxes.drag = Some(ScrollBoxDragState {
          scroll_box_id: id,
          axis: ScrollbarAxis::Vertical,
          button,
          drag_start_mouse: event.y,
          drag_start_thumb_pos: thumb_local,
          thumb_size: thumb.height,
          track_size: bar.height,
          max_scroll: max_scroll_y(state, viewport),
        });
        return true;
      }
    }

    // 命中测试垂直滚动条轨道（翻页）。
    if let Some(bar) = vertical_scrollbar_rect(state, viewport) {
      let physical = scrollbar_physical_rect(bar, canvas.viewport());
      if physical.contains(event.x, event.y) {
        let step = clamped.height as i32;
        if event.y
          < physical.y.saturating_add(
            vertical_thumb_rect(state, viewport)
              .map(|t| t.y.saturating_sub(clamped.y))
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

    // 命中测试水平滚动条滑块。
    if let Some(thumb) = horizontal_thumb_rect(state, viewport) {
      let physical = scrollbar_physical_rect(thumb, canvas.viewport());
      if physical.contains(event.x, event.y) {
        let bar = horizontal_scrollbar_rect(state, viewport).unwrap();
        // 滑块在轨道内的本地偏移（开发者坐标）。
        let thumb_local = thumb.x.saturating_sub(bar.x);
        pool.scroll_boxes.drag = Some(ScrollBoxDragState {
          scroll_box_id: id,
          axis: ScrollbarAxis::Horizontal,
          button,
          drag_start_mouse: event.x,
          drag_start_thumb_pos: thumb_local,
          thumb_size: thumb.width,
          track_size: bar.width,
          max_scroll: max_scroll_x(state, viewport),
        });
        return true;
      }
    }

    // 命中测试水平滚动条轨道（翻页）。
    if let Some(bar) = horizontal_scrollbar_rect(state, viewport) {
      let physical = scrollbar_physical_rect(bar, canvas.viewport());
      if physical.contains(event.x, event.y) {
        let step = clamped.width as i32;
        if event.x
          < physical.x.saturating_add(
            horizontal_thumb_rect(state, viewport)
              .map(|t| t.x.saturating_sub(clamped.x))
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
        // 注意：不更新 drag_start_mouse / drag_start_thumb_pos。
        // 始终以按下时的锚点计算，避免 scroll→thumb→scroll 往返
        // 整数除法累积误差导致滑块漂移/回弹。
        true
      }
      _ => {
        // 拖动期间消费所有鼠标事件。
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

/// 计算考虑滚动条占位后的有效 viewport 尺寸。
pub(crate) fn effective_viewport(state: &ScrollBoxState, viewport: Size) -> Size {
  let rect = clamp_rect(state.options.rect, viewport);
  let mut w = rect.width;
  let mut h = rect.height;
  // Inside 和 ReserveSpace 都需要缩减 viewport，Overlay 不需要。
  if state.options.scrollbar_layout != ScrollbarLayout::Overlay {
    if shows_vertical_scrollbar_raw(state, rect) {
      w = w.saturating_sub(1);
    }
    if shows_horizontal_scrollbar_raw(state, rect) {
      h = h.saturating_sub(1);
    }
  }
  Size {
    width: w,
    height: h,
  }
}

/// 基于原始 clamped_rect 判断垂直滚动条是否应显示（不调用 effective_viewport）。
fn shows_vertical_scrollbar_raw(state: &ScrollBoxState, rect: Rect) -> bool {
  match state.options.scrollbar.vertical {
    ScrollbarVisibility::Always => rect.height > 0,
    ScrollbarVisibility::Auto => state.options.content_height > rect.height,
    ScrollbarVisibility::Never => false,
  }
}

/// 基于原始 clamped_rect 判断水平滚动条是否应显示（不调用 effective_viewport）。
fn shows_horizontal_scrollbar_raw(state: &ScrollBoxState, rect: Rect) -> bool {
  match state.options.scrollbar.horizontal {
    ScrollbarVisibility::Always => rect.width > 0,
    ScrollbarVisibility::Auto => state.options.content_width > rect.width,
    ScrollbarVisibility::Never => false,
  }
}

/// 垂直滚动条是否应显示。
pub(crate) fn shows_vertical_scrollbar(state: &ScrollBoxState, viewport: Size) -> bool {
  let rect = clamp_rect(state.options.rect, viewport);
  shows_vertical_scrollbar_raw(state, rect)
}

/// 水平滚动条是否应显示。
pub(crate) fn shows_horizontal_scrollbar(state: &ScrollBoxState, viewport: Size) -> bool {
  let rect = clamp_rect(state.options.rect, viewport);
  shows_horizontal_scrollbar_raw(state, rect)
}

pub(crate) fn max_scroll_x(state: &ScrollBoxState, viewport: Size) -> u16 {
  state
    .options
    .content_width
    .saturating_sub(effective_viewport(state, viewport).width)
}

pub(crate) fn max_scroll_y(state: &ScrollBoxState, viewport: Size) -> u16 {
  state
    .options
    .content_height
    .saturating_sub(effective_viewport(state, viewport).height)
}

pub(crate) fn clamp_scroll(state: &mut ScrollBoxState, viewport: Size) {
  state.scroll_x = state.scroll_x.min(max_scroll_x(state, viewport));
  state.scroll_y = state.scroll_y.min(max_scroll_y(state, viewport));
}

/// 垂直滚动条轨道矩形（开发者坐标）。
pub(crate) fn vertical_scrollbar_rect(state: &ScrollBoxState, viewport: Size) -> Option<Rect> {
  if !shows_vertical_scrollbar(state, viewport) {
    return None;
  }
  let rect = clamp_rect(state.options.rect, viewport);
  let x = match state.options.scrollbar_layout {
    ScrollbarLayout::Overlay | ScrollbarLayout::Inside => {
      rect.x.saturating_add(rect.width).saturating_sub(1)
    }
    ScrollbarLayout::ReserveSpace => rect.x.saturating_add(rect.width),
  };
  if x >= viewport.width {
    return None;
  }
  Some(Rect {
    x,
    y: rect.y,
    width: 1,
    height: rect.height,
  })
}

/// 水平滚动条轨道矩形（开发者坐标）。
pub(crate) fn horizontal_scrollbar_rect(state: &ScrollBoxState, viewport: Size) -> Option<Rect> {
  if !shows_horizontal_scrollbar(state, viewport) {
    return None;
  }
  let rect = clamp_rect(state.options.rect, viewport);
  let y = match state.options.scrollbar_layout {
    ScrollbarLayout::Overlay | ScrollbarLayout::Inside => {
      rect.y.saturating_add(rect.height).saturating_sub(1)
    }
    ScrollbarLayout::ReserveSpace => rect.y.saturating_add(rect.height),
  };
  if y >= viewport.height {
    return None;
  }
  // ReserveSpace 和 Inside 下水平滚动条不延伸至垂直滚动条占位列。
  let width = if state.options.scrollbar_layout != ScrollbarLayout::Overlay
    && shows_vertical_scrollbar(state, viewport)
  {
    rect.width
  } else {
    rect.width
  };
  Some(Rect {
    x: rect.x,
    y,
    width,
    height: 1,
  })
}

/// 垂直滚动条滑块矩形（开发者坐标）。
pub(crate) fn vertical_thumb_rect(state: &ScrollBoxState, viewport: Size) -> Option<Rect> {
  let bar = vertical_scrollbar_rect(state, viewport)?;
  let max_scroll = max_scroll_y(state, viewport);
  let height = bar.height;
  let thumb_height = if max_scroll == 0 {
    height
  } else {
    ((height as u32 * height as u32) / state.options.content_height.max(1) as u32)
      .max(state.options.scrollbar_style.minimum_thumb_height as u32)
      .min(height as u32) as u16
  };
  let travel = height.saturating_sub(thumb_height);
  let thumb_y = if max_scroll == 0 {
    0
  } else {
    (state.scroll_y as u32 * travel as u32 / max_scroll as u32) as u16
  };
  Some(Rect {
    x: bar.x,
    y: bar.y.saturating_add(thumb_y),
    width: 1,
    height: thumb_height,
  })
}

/// 水平滚动条滑块矩形（开发者坐标）。
pub(crate) fn horizontal_thumb_rect(state: &ScrollBoxState, viewport: Size) -> Option<Rect> {
  let bar = horizontal_scrollbar_rect(state, viewport)?;
  let max_scroll = max_scroll_x(state, viewport);
  let width = bar.width;
  let thumb_width = if max_scroll == 0 {
    width
  } else {
    ((width as u32 * width as u32) / state.options.content_width.max(1) as u32)
      .max(state.options.scrollbar_style.minimum_thumb_height as u32)
      .min(width as u32) as u16
  };
  let travel = width.saturating_sub(thumb_width);
  let thumb_x = if max_scroll == 0 {
    0
  } else {
    (state.scroll_x as u32 * travel as u32 / max_scroll as u32) as u16
  };
  Some(Rect {
    x: bar.x.saturating_add(thumb_x),
    y: bar.y,
    width: thumb_width,
    height: 1,
  })
}

/// 将开发者坐标下的滚动条矩形转换到物理坐标。
fn scrollbar_physical_rect(rect: Rect, viewport: Rect) -> Rect {
  Rect {
    x: viewport.x.saturating_add(rect.x),
    y: viewport.y.saturating_add(rect.y),
    width: rect.width,
    height: rect.height,
  }
}

/// 寻找鼠标下方最顶层的 ScrollBox（包含滚动条区域）。
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
        let content_rect = clamp_rect(state.options.rect, viewport_size);
        let physical_content = scrollbar_physical_rect(content_rect, canvas.viewport());
        if physical_content.contains(x, y) {
          return Some(*id);
        }
        // 同时检查滚动条区域（ReserveSpace 下会超出内容矩形）。
        if let Some(v_bar) = vertical_scrollbar_rect(state, viewport_size) {
          let p = scrollbar_physical_rect(v_bar, canvas.viewport());
          if p.contains(x, y) {
            return Some(*id);
          }
        }
        if let Some(h_bar) = horizontal_scrollbar_rect(state, viewport_size) {
          let p = scrollbar_physical_rect(h_bar, canvas.viewport());
          if p.contains(x, y) {
            return Some(*id);
          }
        }
        None
      }
      _ => None,
    })
    .next()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{
    CanvasService, MouseButton, ScrollDirection, SliceOptions, SliceService,
  };

  #[test]
  fn create_rejects_zero_wheel_step_and_allows_horizontal_overflow() {
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    // 水平溢出现在允许。
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
    // wheel_step = 0 仍然拒绝。
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

    // 垂直夹紧。
    assert!(service.scroll_to(&mut pool, id, 0, 99, &layout));
    assert_eq!(service.scroll_y(&pool, id), Some(6));
    assert!(service.scroll_by(&mut pool, id, 0, -10, &layout));
    assert_eq!(service.scroll_y(&pool, id), Some(0));
    assert!(service.scroll_to_bottom(&mut pool, id, &layout));
    assert_eq!(service.scroll_y(&pool, id), Some(6));

    // 水平夹紧。
    assert!(service.scroll_to(&mut pool, id, 99, 0, &layout));
    assert_eq!(service.scroll_x(&pool, id), Some(12));
    assert!(service.scroll_by(&mut pool, id, -20, 0, &layout));
    assert_eq!(service.scroll_x(&pool, id), Some(0));
    assert!(service.scroll_to_right(&mut pool, id, &layout));
    assert_eq!(service.scroll_x(&pool, id), Some(12));
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
    assert_eq!(visible.width, 8);
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

    // 内容 → 视口。
    assert_eq!(
      service.content_to_viewport_point(&pool, id, 3, 5),
      Some((0, 0))
    );
    assert_eq!(service.content_to_viewport_point(&pool, id, 0, 0), None); // 内容坐标 < scroll → None

    // 视口 → 内容。
    assert_eq!(
      service.viewport_to_content_point(&pool, id, 0, 0),
      Some((3, 5))
    );
    assert_eq!(
      service.viewport_to_content_point(&pool, id, 1, 1),
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

    // 再次滚动到相同位置不应发射事件。
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
    canvas.prepare(&pool, &layout);

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
    canvas.prepare(&pool, &layout);
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
    canvas.prepare(&pool, &layout);

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
    canvas.prepare(&pool, &layout);

    // 垂直滚动条在 x=7（Overlay 模式的最右侧列）。滑块初始在顶部。
    // 在滑块区域按下鼠标 → 开始拖动。
    let pressed = service.route_mouse_event(
      &mut pool,
      &canvas,
      &layout,
      MouseEvent {
        kind: MouseEventKind::Press,
        button: Some(MouseButton::Left),
        scroll: None,
        x: 7, // 滚动条列
        y: 0, // 滑块顶部
      },
    );
    assert!(pressed, "press on thumb should be consumed");
    assert!(pool.scroll_boxes.drag.is_some());

    // 释放 → 结束拖动。
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
    canvas.prepare(&pool, &layout);

    // 点击轨道底部（滑块下方）→ 向下翻页。
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
        y: 3, // 轨道底部，滑块下方
      },
    );
    assert!(pressed);
    // 验证已向下滚动。
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
    canvas.prepare(&pool, &layout);

    // 两个方向都被 blocking。
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
    // ReserveSpace: 内容高度 10 > viewport 5 → 垂直滚动条显示 → 宽度减 1
    // 内容宽度 20 > viewport 10 → 水平滚动条显示 → 高度减 1
    let eff = effective_viewport(&state, viewport);
    assert_eq!(eff.width, 9);
    assert_eq!(eff.height, 4);
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
  fn overlay_scrollbar_does_not_reduce_visible_content_queries() {
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
        width: 10,
        height: 5
      })
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
    // content_height=5 == rect.height=5 → 垂直滚动条不显示
    // content_width=20 > rect.width=10 → 水平滚动条显示 → 高度减 1，宽度不变
    assert_eq!(max_scroll_x(&state, viewport), 10); // 20 - 10
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
        scrollbar_layout: ScrollbarLayout::Inside, // 默认
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
    // 垂直滚动条显示 → 宽度减 1，高度不变。
    let eff = effective_viewport(&state, viewport);
    assert_eq!(eff.width, 9);
    assert_eq!(eff.height, 5);
    assert_eq!(max_scroll_y(&state, viewport), 5); // 10 - 5
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
            track_char: '中',         // CJK 宽 2，应退回默认 '│'
            thumb_char: '\u{200D}',   // ZWJ 宽 0，应退回默认 '█'
            h_track_char: '━',        // 宽 1，OK
            h_thumb_char: '\u{200D}', // 宽 0，应退回默认 '█'
            ..Default::default()
          },
          ..Default::default()
        },
      )
      .unwrap();
    let state = pool.scroll_boxes.boxes.get(&id).unwrap();
    let style = &state.options.scrollbar_style;
    assert_eq!(style.track_char, '│'); // 退回默认
    assert_eq!(style.thumb_char, '█'); // 退回默认
    assert_eq!(style.h_track_char, '━'); // 合法宽度
    assert_eq!(style.h_thumb_char, '█'); // 退回默认
  }
}
