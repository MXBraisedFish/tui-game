use std::collections::HashMap;

use super::surface::SurfaceId;
use super::ui::UiObjectPool;
use super::{
  CanvasService, LayoutService, MouseEvent, MouseEventKind, Rect, ScrollDirection, Size, TextColor,
  TextStyle,
};

/// 可滚动绘制面唯一标识。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScrollBoxId(pub u64);

/// 溢出处理方式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
  Hidden,
  Auto,
}

/// 滚动条显示策略。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarVisibility {
  Auto,
  Always,
  Never,
}

/// 滚动条策略。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollbarPolicy {
  pub vertical: ScrollbarVisibility,
}

impl Default for ScrollbarPolicy {
  fn default() -> Self {
    Self {
      vertical: ScrollbarVisibility::Auto,
    }
  }
}

/// 滚动条样式。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollbarStyle {
  pub track_char: char,
  pub thumb_char: char,
  pub track_style: TextStyle,
  pub thumb_style: TextStyle,
}

impl Default for ScrollbarStyle {
  fn default() -> Self {
    Self {
      track_char: '│',
      thumb_char: '█',
      track_style: TextStyle {
        foreground: Some(TextColor::Terminal(super::TerminalColor::BrightBlack)),
        ..Default::default()
      },
      thumb_style: TextStyle {
        foreground: Some(TextColor::Terminal(super::TerminalColor::BrightWhite)),
        ..Default::default()
      },
    }
  }
}

/// 可滚动绘制面配置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollBoxOptions {
  pub rect: Rect,
  pub content_width: u16,
  pub content_height: u16,
  pub overflow_y: Overflow,
  pub overflow_x: Overflow,
  pub scrollbar: ScrollbarPolicy,
  pub scrollbar_style: ScrollbarStyle,
  pub visible: bool,
  pub opaque: bool,
  pub mouse_wheel: bool,
  pub wheel_step: u16,
}

impl Default for ScrollBoxOptions {
  fn default() -> Self {
    Self {
      rect: Rect::default(),
      content_width: 0,
      content_height: 0,
      overflow_y: Overflow::Auto,
      overflow_x: Overflow::Hidden,
      scrollbar: ScrollbarPolicy::default(),
      scrollbar_style: ScrollbarStyle::default(),
      visible: true,
      opaque: true,
      mouse_wheel: true,
      wheel_step: 3,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScrollBoxState {
  pub options: ScrollBoxOptions,
  pub scroll_x: u16,
  pub scroll_y: u16,
}

pub(crate) struct ScrollBoxObjects {
  pub next_id: u64,
  pub boxes: HashMap<ScrollBoxId, ScrollBoxState>,
}

impl ScrollBoxObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      boxes: HashMap::new(),
    }
  }
}

/// 可滚动绘制面服务。
pub struct ScrollBoxService;

impl ScrollBoxService {
  pub fn new() -> Self {
    Self
  }

  pub fn create(&self, pool: &mut UiObjectPool, options: ScrollBoxOptions) -> Option<ScrollBoxId> {
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
    let _ = self.viewport_size(pool, id, layout)?;
    Some(0)
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

  pub fn scroll_to(
    &self,
    pool: &mut UiObjectPool,
    id: ScrollBoxId,
    _x: u16,
    y: u16,
    layout: &LayoutService,
  ) -> bool {
    let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
      return false;
    };
    state.scroll_x = 0;
    state.scroll_y = y.min(max_scroll_y(state, layout.developer_size()));
    true
  }

  pub fn scroll_by(
    &self,
    pool: &mut UiObjectPool,
    id: ScrollBoxId,
    _dx: i32,
    dy: i32,
    layout: &LayoutService,
  ) -> bool {
    let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
      return false;
    };
    let next = (state.scroll_y as i32).saturating_add(dy).max(0) as u16;
    state.scroll_x = 0;
    state.scroll_y = next.min(max_scroll_y(state, layout.developer_size()));
    true
  }

  pub fn scroll_to_top(&self, pool: &mut UiObjectPool, id: ScrollBoxId) -> bool {
    let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
      return false;
    };
    state.scroll_x = 0;
    state.scroll_y = 0;
    true
  }

  pub fn scroll_to_bottom(
    &self,
    pool: &mut UiObjectPool,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) -> bool {
    let Some(state) = pool.scroll_boxes.boxes.get_mut(&id) else {
      return false;
    };
    state.scroll_x = 0;
    state.scroll_y = max_scroll_y(state, layout.developer_size());
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

  pub(crate) fn route_mouse_event(
    &self,
    pool: &mut UiObjectPool,
    canvas: &CanvasService,
    layout: &LayoutService,
    event: MouseEvent,
  ) -> bool {
    if event.kind != MouseEventKind::Scroll {
      return false;
    }
    let Some(id) = canvas.top_scroll_box_at(event.x, event.y) else {
      return false;
    };
    let Some(state) = pool.scroll_boxes.boxes.get(&id) else {
      return false;
    };
    if !state.options.mouse_wheel || max_scroll_y(state, layout.developer_size()) == 0 {
      return false;
    }
    let dy = match event.scroll {
      Some(ScrollDirection::Up) => -(state.options.wheel_step as i32),
      Some(ScrollDirection::Down) => state.options.wheel_step as i32,
      _ => return false,
    };
    self.scroll_by(pool, id, 0, dy, layout)
  }
}

fn valid_options(options: &ScrollBoxOptions) -> bool {
  options.overflow_x == Overflow::Hidden && options.wheel_step > 0
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

pub(crate) fn max_scroll_y(state: &ScrollBoxState, viewport: Size) -> u16 {
  state
    .options
    .content_height
    .saturating_sub(clamp_rect(state.options.rect, viewport).height)
}

pub(crate) fn clamp_scroll(state: &mut ScrollBoxState, viewport: Size) {
  state.scroll_x = 0;
  state.scroll_y = state.scroll_y.min(max_scroll_y(state, viewport));
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{CanvasService, ScrollDirection, SliceOptions, SliceService};

  #[test]
  fn create_rejects_horizontal_overflow_and_zero_wheel_step() {
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
        .is_none()
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
  fn scroll_is_clamped_to_content_height() {
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
}
