mod state;
mod types;

pub(crate) use self::state::HitAreaObjects;
use self::state::{HitAreaState, HitSnapshot, PressState};
pub use self::types::{HitAreaEvent, HitAreaId, HitAreaOptions};
use crate::host_engine::services::ui::UiObjectPool;
use crate::host_engine::services::{
  CanvasService, MouseButton, MouseEvent, MouseEventKind, Rect, SliceId, TextInputService,
};

/// 点击区域服务，管理鼠标交互区域
pub struct HitAreaService;

impl HitAreaService {
  pub fn new() -> Self {
    Self
  }

  /// 创建一个新的点击区域
  pub fn create(&self, pool: &mut UiObjectPool, options: HitAreaOptions) -> HitAreaId {
    let id = HitAreaId(pool.hit_areas.next_id);
    pool.hit_areas.next_id += 1;
    pool
      .hit_areas
      .areas
      .insert(id, HitAreaState { hit: None, options });
    id
  }

  /// 移除一个点击区域
  pub fn remove(&self, pool: &mut UiObjectPool, id: HitAreaId) -> bool {
    if pool.hit_areas.areas.remove(&id).is_none() {
      return false;
    }
    if pool.hit_areas.hovered == Some(id) {
      pool.hit_areas.hovered = None;
    }
    pool.hit_areas.pressed.retain(|_, state| state.id != id);
    pool.events.retain(|event| event.hit_area_id() != Some(id));
    true
  }

  /// 检查点击区域是否存在
  pub fn exists(&self, pool: &UiObjectPool, id: HitAreaId) -> bool {
    pool.hit_areas.areas.contains_key(&id)
  }

  /// 检查点击区域是否被悬停
  pub fn is_hovered(&self, pool: &UiObjectPool, id: HitAreaId) -> bool {
    pool.hit_areas.areas.contains_key(&id) && pool.hit_areas.hovered == Some(id)
  }

  /// 检查指定按键的点击区域是否被按下
  pub fn is_pressed(&self, pool: &UiObjectPool, id: HitAreaId, button: MouseButton) -> bool {
    pool
      .hit_areas
      .pressed
      .get(&button)
      .is_some_and(|pressed| pressed.id == id)
  }

  /// 获取当前指针在视口内的位置
  pub fn pointer_position(&self, pool: &UiObjectPool) -> Option<(u16, u16)> {
    pool.hit_areas.pointer
  }

  /// 获取指针在指定点击区域内的本地坐标
  pub fn local_pointer_position(&self, pool: &UiObjectPool, id: HitAreaId) -> Option<(u16, u16)> {
    let (x, y) = pool.hit_areas.physical_pointer?;
    let hit = pool.hit_areas.areas.get(&id)?.hit?;
    hit
      .rect
      .contains(x, y)
      .then(|| (x - hit.rect.x, y - hit.rect.y))
  }

  /// 渲染点击区域的基础命中矩形
  pub fn render(
    &self,
    pool: &mut UiObjectPool,
    id: HitAreaId,
    rect: Rect,
    canvas: &CanvasService,
  ) -> bool {
    self.render_resolved(pool, id, canvas.base_hit_rect(rect))
  }

  /// 在指定切片上渲染点击区域的命中矩形
  pub fn render_on(
    &self,
    pool: &mut UiObjectPool,
    id: HitAreaId,
    slice: SliceId,
    rect: Rect,
    canvas: &CanvasService,
  ) -> bool {
    self.render_resolved(pool, id, canvas.slice_hit_rect(slice, rect))
  }

  pub(crate) fn render_host(
    &self,
    pool: &mut UiObjectPool,
    id: HitAreaId,
    rect: Rect,
    canvas: &CanvasService,
  ) -> bool {
    self.render_resolved(pool, id, canvas.host_hit_rect(rect))
  }

  fn render_resolved(
    &self,
    pool: &mut UiObjectPool,
    id: HitAreaId,
    resolved: Option<(Rect, (u16, u16), usize)>,
  ) -> bool {
    if !pool.hit_areas.areas.contains_key(&id) {
      return false;
    }
    let order = pool.next_render_order();
    let state = pool.hit_areas.areas.get_mut(&id).unwrap();
    state.hit = resolved.map(|(rect, origin, surface_rank)| HitSnapshot {
      rect,
      order,
      origin,
      surface_rank,
    });
    true
  }

  pub(crate) fn route_mouse_event(
    &self,
    pool: &mut UiObjectPool,
    text_input: &mut TextInputService,
    canvas: &CanvasService,
    event: MouseEvent,
  ) -> bool {
    if event.kind == MouseEventKind::Hold {
      let captured = event
        .button
        .is_some_and(|button| pool.hit_areas.pressed.contains_key(&button));
      return text_input.route_mouse_event(pool, event) || captured;
    }
    if event.kind == MouseEventKind::Scroll {
      return text_input.route_mouse_event(pool, event);
    }

    let hit = pool.hit_areas.hit(event.x, event.y);
    let text_order = text_input.mouse_hit_order(pool, event.x, event.y);
    let top_hit = hit.filter(|(_, order)| text_order.is_none_or(|text| *order > text));
    pool.hit_areas.pointer = canvas.viewport_point(event.x, event.y);
    pool.hit_areas.physical_pointer = Some((event.x, event.y));
    self.update_hover(pool, top_hit.map(|(id, _)| id), event.x, event.y);

    match event.kind {
      MouseEventKind::Move => top_hit.is_some() || text_order.is_some(),
      MouseEventKind::Press => {
        let Some(button) = event.button else {
          return false;
        };
        if let Some((id, _)) = top_hit {
          text_input.push_pressed_outside(pool);
          pool.hit_areas.pressed.insert(
            button,
            PressState {
              id,
              last_x: event.x,
              last_y: event.y,
            },
          );
          let (x, y) = event_point(pool, id, event.x, event.y);
          pool.push_hit_event(HitAreaEvent::Press { id, button, x, y });
          true
        } else if text_order.is_some() {
          text_input.route_mouse_event(pool, event);
          true
        } else {
          text_input.route_mouse_event(pool, event)
        }
      }
      MouseEventKind::Drag => {
        let Some(button) = event.button else {
          return false;
        };
        if let Some(press) = pool.hit_areas.pressed.get_mut(&button) {
          let (id, dx, dy) = (
            press.id,
            event.x as i32 - press.last_x as i32,
            event.y as i32 - press.last_y as i32,
          );
          press.last_x = event.x;
          press.last_y = event.y;
          if pool.hit_areas.areas[&id].options.drag {
            let (x, y) = event_point(pool, id, event.x, event.y);
            pool.push_hit_event(HitAreaEvent::Drag {
              id,
              button,
              x,
              y,
              dx,
              dy,
            });
          }
          true
        } else {
          text_input.route_mouse_event(pool, event) || top_hit.is_some() || text_order.is_some()
        }
      }
      MouseEventKind::Release => {
        let Some(button) = event.button else {
          return false;
        };
        let pressed = pool.hit_areas.pressed.remove(&button);
        let text_consumed = text_input.route_mouse_event(pool, event);
        if let Some((id, _)) = top_hit {
          let (x, y) = event_point(pool, id, event.x, event.y);
          pool.push_hit_event(HitAreaEvent::Release { id, button, x, y });
          if pressed.is_some_and(|press| press.id == id) {
            pool.push_hit_event(HitAreaEvent::Click { id, button, x, y });
          }
          true
        } else {
          text_consumed || pressed.is_some() || text_order.is_some()
        }
      }
      _ => false,
    }
  }

  pub(crate) fn focus_lost(&self, pool: &mut UiObjectPool) {
    if let (Some(id), Some((x, y))) = (
      pool.hit_areas.hovered.take(),
      pool.hit_areas.physical_pointer,
    ) {
      let (x, y) = event_point(pool, id, x, y);
      pool.push_hit_event(HitAreaEvent::HoverLeave { id, x, y });
    }
    pool.hit_areas.pressed.clear();
  }

  pub(crate) fn deactivate(&self, pool: &mut UiObjectPool) {
    pool.hit_areas.hovered = None;
    pool.hit_areas.pressed.clear();
    pool.hit_areas.pointer = None;
    pool.hit_areas.physical_pointer = None;
    for state in pool.hit_areas.areas.values_mut() {
      state.hit = None;
    }
    pool.events.clear();
  }

  fn update_hover(&self, pool: &mut UiObjectPool, current: Option<HitAreaId>, x: u16, y: u16) {
    let previous = pool.hit_areas.hovered;
    match (previous, current) {
      (Some(old), Some(new)) if old == new => {
        if pool.hit_areas.areas[&new].options.hover_move {
          let (x, y) = event_point(pool, new, x, y);
          pool.push_hit_event(HitAreaEvent::HoverMove { id: new, x, y });
        }
      }
      (Some(old), Some(new)) => {
        let old_point = event_point(pool, old, x, y);
        let new_point = event_point(pool, new, x, y);
        pool.push_hit_event(HitAreaEvent::HoverLeave {
          id: old,
          x: old_point.0,
          y: old_point.1,
        });
        pool.push_hit_event(HitAreaEvent::HoverEnter {
          id: new,
          x: new_point.0,
          y: new_point.1,
        });
      }
      (Some(old), None) => {
        let (x, y) = event_point(pool, old, x, y);
        pool.push_hit_event(HitAreaEvent::HoverLeave { id: old, x, y });
      }
      (None, Some(new)) => {
        let (x, y) = event_point(pool, new, x, y);
        pool.push_hit_event(HitAreaEvent::HoverEnter { id: new, x, y });
      }
      (None, None) => {}
    }
    pool.hit_areas.hovered = current;
  }
}

// 计算相对于点击区域原点（切片偏移）的本地坐标
fn event_point(pool: &UiObjectPool, id: HitAreaId, x: u16, y: u16) -> (u16, u16) {
  pool.hit_areas.areas[&id]
    .hit
    .map(|hit| {
      (
        x.saturating_sub(hit.origin.0),
        y.saturating_sub(hit.origin.1),
      )
    })
    .unwrap_or((x, y))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{
    CanvasService, LayoutService, SliceLength, SliceOptions, SliceRect, SliceService,
    TextInputEvent, TextInputMode, TextInputOptions, TextInputRenderParams, UiEvent,
  };

  fn mouse(kind: MouseEventKind, button: Option<MouseButton>, x: u16, y: u16) -> MouseEvent {
    MouseEvent {
      kind,
      button,
      scroll: None,
      x,
      y,
    }
  }

  fn events(pool: &mut UiObjectPool) -> Vec<UiEvent> {
    std::iter::from_fn(|| pool.pop_event()).collect()
  }

  fn rect(x: u16, y: u16, width: u16, height: u16) -> Rect {
    Rect {
      x,
      y,
      width,
      height,
    }
  }

  #[test]
  fn lifecycle_zero_rect_and_missing_render_are_consistent() {
    let service = HitAreaService::new();
    let mut text_input = TextInputService::new();
    let mut pool = UiObjectPool::new();
    let canvas = CanvasService::new();
    let first = service.create(&mut pool, HitAreaOptions::default());
    let second = service.create(&mut pool, HitAreaOptions::default());
    assert_eq!((first, second), (HitAreaId(1), HitAreaId(2)));
    assert!(service.exists(&pool, first));
    assert!(service.render(&mut pool, first, rect(0, 0, 0, 2), &canvas));
    assert!(!service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Move, None, 0, 0),
    ));

    service.render(&mut pool, first, rect(0, 0, 3, 2), &canvas);
    assert!(service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Move, None, 1, 1),
    ));
    pool.begin_render();
    assert!(!service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Move, None, 1, 1),
    ));
    assert!(service.remove(&mut pool, first));
    assert!(!service.remove(&mut pool, first));
  }

  #[test]
  fn overlap_hover_and_click_use_last_rendered_area() {
    let service = HitAreaService::new();
    let mut text_input = TextInputService::new();
    let mut pool = UiObjectPool::new();
    let canvas = CanvasService::new();
    let a = service.create(&mut pool, HitAreaOptions::default());
    let b = service.create(&mut pool, HitAreaOptions::default());
    service.render(&mut pool, a, rect(0, 0, 4, 4), &canvas);
    service.render(&mut pool, b, rect(2, 0, 4, 4), &canvas);

    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Move, None, 1, 1),
    );
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Move, None, 2, 1),
    );
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Press, Some(MouseButton::Right), 2, 1),
    );
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Release, Some(MouseButton::Right), 2, 1),
    );

    let events = events(&mut pool);
    assert!(events.contains(&UiEvent::HitArea(HitAreaEvent::HoverLeave {
      id: a,
      x: 2,
      y: 1,
    })));
    assert!(events.contains(&UiEvent::HitArea(HitAreaEvent::HoverEnter {
      id: b,
      x: 2,
      y: 1,
    })));
    assert!(events.contains(&UiEvent::HitArea(HitAreaEvent::Click {
      id: b,
      button: MouseButton::Right,
      x: 2,
      y: 1,
    })));
  }

  #[test]
  fn slice_surface_beats_later_base_hit_and_reports_local_coordinates() {
    let service = HitAreaService::new();
    let slices = SliceService::new();
    let mut text_input = TextInputService::new();
    let mut pool = UiObjectPool::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    layout.set_developer_viewport(rect(3, 2, 10, 6));
    let slice = slices
      .create(
        &mut pool,
        SliceOptions {
          rect: SliceRect {
            x: 2,
            y: 1,
            width: SliceLength::Fixed(4),
            height: SliceLength::Fixed(3),
          },
          ..Default::default()
        },
      )
      .unwrap();
    let slice_area = service.create(&mut pool, HitAreaOptions::default());
    let base_area = service.create(&mut pool, HitAreaOptions::default());
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);

    service.render_on(&mut pool, slice_area, slice, rect(1, 1, 2, 1), &canvas);
    service.render(&mut pool, base_area, rect(3, 2, 2, 1), &canvas);
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Move, None, 6, 4),
    );

    assert_eq!(
      events(&mut pool),
      vec![UiEvent::HitArea(HitAreaEvent::HoverEnter {
        id: slice_area,
        x: 1,
        y: 1,
      })]
    );
    assert_eq!(service.pointer_position(&pool), Some((3, 2)));
    assert_eq!(
      service.local_pointer_position(&pool, slice_area),
      Some((0, 0))
    );
  }

  #[test]
  fn default_continuous_events_are_query_only() {
    let service = HitAreaService::new();
    let mut text_input = TextInputService::new();
    let mut pool = UiObjectPool::new();
    let canvas = CanvasService::new();
    let id = service.create(&mut pool, HitAreaOptions::default());
    service.render(&mut pool, id, rect(10, 5, 4, 3), &canvas);

    assert_eq!(service.pointer_position(&pool), None);
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Move, None, 11, 6),
    );
    events(&mut pool);
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Move, None, 12, 6),
    );
    assert!(events(&mut pool).is_empty());
    assert!(service.is_hovered(&pool, id));
    assert_eq!(service.pointer_position(&pool), Some((12, 6)));
    assert_eq!(service.local_pointer_position(&pool, id), Some((2, 1)));

    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Press, Some(MouseButton::Left), 12, 6),
    );
    events(&mut pool);
    assert!(service.is_pressed(&pool, id, MouseButton::Left));
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Drag, Some(MouseButton::Left), 20, 6),
    );
    assert!(
      events(&mut pool)
        .iter()
        .all(|event| !matches!(event, UiEvent::HitArea(HitAreaEvent::Drag { .. })))
    );
    assert!(service.is_pressed(&pool, id, MouseButton::Left));
    assert_eq!(service.local_pointer_position(&pool, id), None);
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Release, Some(MouseButton::Left), 20, 6),
    );
    assert!(!service.is_pressed(&pool, id, MouseButton::Left));
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Move, None, 5, 5),
    );
    assert_eq!(service.local_pointer_position(&pool, id), None);
  }

  #[test]
  fn subscribed_hover_move_is_queued() {
    let service = HitAreaService::new();
    let mut text_input = TextInputService::new();
    let mut pool = UiObjectPool::new();
    let canvas = CanvasService::new();
    let id = service.create(
      &mut pool,
      HitAreaOptions {
        hover_move: true,
        drag: false,
      },
    );
    service.render(&mut pool, id, rect(0, 0, 3, 3), &canvas);
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Move, None, 1, 1),
    );
    events(&mut pool);
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Move, None, 2, 1),
    );
    assert_eq!(
      events(&mut pool),
      vec![UiEvent::HitArea(HitAreaEvent::HoverMove { id, x: 2, y: 1 })]
    );
  }

  #[test]
  fn all_mouse_buttons_have_press_release_and_click() {
    for button in [MouseButton::Left, MouseButton::Middle, MouseButton::Right] {
      let service = HitAreaService::new();
      let mut text_input = TextInputService::new();
      let mut pool = UiObjectPool::new();
      let canvas = CanvasService::new();
      let id = service.create(&mut pool, HitAreaOptions::default());
      service.render(&mut pool, id, rect(0, 0, 3, 3), &canvas);
      service.route_mouse_event(
        &mut pool,
        &mut text_input,
        &canvas,
        mouse(MouseEventKind::Press, Some(button), 1, 1),
      );
      service.route_mouse_event(
        &mut pool,
        &mut text_input,
        &canvas,
        mouse(MouseEventKind::Release, Some(button), 1, 1),
      );
      let events = events(&mut pool);
      let kinds = events
        .iter()
        .filter_map(|event| match event {
          UiEvent::HitArea(HitAreaEvent::Press { button: actual, .. }) if *actual == button => {
            Some("press")
          }
          UiEvent::HitArea(HitAreaEvent::Release { button: actual, .. }) if *actual == button => {
            Some("release")
          }
          UiEvent::HitArea(HitAreaEvent::Click { button: actual, .. }) if *actual == button => {
            Some("click")
          }
          _ => None,
        })
        .collect::<Vec<_>>();
      assert_eq!(kinds, ["press", "release", "click"]);
    }
  }

  #[test]
  fn drag_is_captured_and_click_requires_matching_release_target() {
    let service = HitAreaService::new();
    let mut text_input = TextInputService::new();
    let mut pool = UiObjectPool::new();
    let canvas = CanvasService::new();
    let a = service.create(
      &mut pool,
      HitAreaOptions {
        hover_move: false,
        drag: true,
      },
    );
    let b = service.create(&mut pool, HitAreaOptions::default());
    service.render(&mut pool, a, rect(0, 0, 3, 3), &canvas);
    service.render(&mut pool, b, rect(5, 0, 3, 3), &canvas);
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Press, Some(MouseButton::Left), 1, 1),
    );
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Drag, Some(MouseButton::Left), 4, 2),
    );
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Drag, Some(MouseButton::Left), 6, 1),
    );
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Release, Some(MouseButton::Left), 6, 1),
    );
    let events = events(&mut pool);
    assert!(events.contains(&UiEvent::HitArea(HitAreaEvent::Drag {
      id: a,
      button: MouseButton::Left,
      x: 4,
      y: 2,
      dx: 3,
      dy: 1,
    })));
    assert!(events.contains(&UiEvent::HitArea(HitAreaEvent::Drag {
      id: a,
      button: MouseButton::Left,
      x: 6,
      y: 1,
      dx: 2,
      dy: -1,
    })));
    assert!(
      !events
        .iter()
        .any(|event| matches!(event, UiEvent::HitArea(HitAreaEvent::Click { .. })))
    );
  }

  #[test]
  fn later_text_input_blocks_hit_area_and_hit_area_can_report_pressed_outside() {
    let hit_area = HitAreaService::new();
    let mut text_input = TextInputService::new();
    let mut pool = UiObjectPool::new();
    let mut canvas = CanvasService::new();
    let area = hit_area.create(&mut pool, HitAreaOptions::default());
    let input = text_input.create(
      &mut pool,
      TextInputOptions {
        mode: TextInputMode::SingleLine,
        mouse: true,
        ..Default::default()
      },
    );
    hit_area.render(&mut pool, area, rect(0, 0, 5, 1), &canvas);
    text_input.render(
      &mut pool,
      input,
      &TextInputRenderParams {
        rect: rect(0, 0, 5, 1),
        ..Default::default()
      },
      &mut canvas,
    );
    hit_area.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Press, Some(MouseButton::Left), 1, 0),
    );
    assert_eq!(
      events(&mut pool),
      vec![UiEvent::TextInput(TextInputEvent::Pressed { id: input })]
    );

    pool.begin_render();
    text_input.render(
      &mut pool,
      input,
      &TextInputRenderParams {
        rect: rect(0, 0, 5, 1),
        ..Default::default()
      },
      &mut canvas,
    );
    hit_area.render(&mut pool, area, rect(0, 0, 5, 1), &canvas);
    text_input.focus(&mut pool, input);
    events(&mut pool);
    hit_area.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Press, Some(MouseButton::Left), 1, 0),
    );
    let events = events(&mut pool);
    assert!(
      events.contains(&UiEvent::TextInput(TextInputEvent::PressedOutside {
        id: input
      }))
    );
    assert!(events.contains(&UiEvent::HitArea(HitAreaEvent::Press {
      id: area,
      button: MouseButton::Left,
      x: 1,
      y: 0,
    })));
  }

  #[test]
  fn focus_lost_leaves_hover_and_cancels_click() {
    let service = HitAreaService::new();
    let mut text_input = TextInputService::new();
    let mut pool = UiObjectPool::new();
    let canvas = CanvasService::new();
    let id = service.create(&mut pool, HitAreaOptions::default());
    service.render(&mut pool, id, rect(0, 0, 3, 3), &canvas);
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Press, Some(MouseButton::Middle), 1, 1),
    );
    events(&mut pool);
    service.focus_lost(&mut pool);
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Release, Some(MouseButton::Middle), 1, 1),
    );
    let events = events(&mut pool);
    assert_eq!(
      events[0],
      UiEvent::HitArea(HitAreaEvent::HoverLeave { id, x: 1, y: 1 })
    );
    assert!(
      !events
        .iter()
        .any(|event| matches!(event, UiEvent::HitArea(HitAreaEvent::Click { .. })))
    );
  }

  #[test]
  fn deactivate_silently_clears_hits_events_and_capture() {
    let service = HitAreaService::new();
    let mut text_input = TextInputService::new();
    let mut pool = UiObjectPool::new();
    let canvas = CanvasService::new();
    let id = service.create(&mut pool, HitAreaOptions::default());
    service.render(&mut pool, id, rect(0, 0, 3, 3), &canvas);
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Press, Some(MouseButton::Left), 1, 1),
    );
    service.deactivate(&mut pool);
    service.route_mouse_event(
      &mut pool,
      &mut text_input,
      &canvas,
      mouse(MouseEventKind::Release, Some(MouseButton::Left), 1, 1),
    );
    assert!(events(&mut pool).is_empty());
  }
}
