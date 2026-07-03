use std::ops::Range;
use std::time::{Duration, Instant};

use super::buffer::TextBuffer;
use super::render::{fill_input_background, render_multi_line, render_single_line};
use super::state::{ActiveTextInput, DragSelection, TextInputActive, TextInputState};
use super::types::{
  TextInputCursorShape, TextInputEvent, TextInputId, TextInputMode, TextInputOptions,
  TextInputRenderParams, TextSurface,
};
use crate::host_engine::services::ui::UiObjectPool;
use crate::host_engine::services::{CanvasService, SliceId};

const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

/// 文本输入服务：管理输入焦点、光标、选区、键盘和鼠标路由及渲染。
pub struct TextInputService {
  pub(super) active: TextInputActive,
  pub(super) drag: Option<DragSelection>,
  pub(super) cursor_blink_started: Instant,
}

impl TextInputService {
  pub fn new() -> Self {
    Self {
      active: TextInputActive::Inactive,
      drag: None,
      cursor_blink_started: Instant::now(),
    }
  }

  /// 在对象池中创建一个新的文本输入组件。
  pub fn create(&self, pool: &mut UiObjectPool, options: TextInputOptions) -> TextInputId {
    let objects = &mut pool.text_inputs;
    let id = TextInputId(objects.next_input_id);
    objects.next_input_id += 1;
    objects.inputs.insert(
      id,
      TextInputState {
        buffer: TextBuffer::new(options.initial_text, options.max_chars, options.mode),
        mode: options.mode,
        mouse: options.mouse,
        hit: None,
        pending_cursor: None,
        visual_line: None,
      },
    );
    id
  }

  /// 移除文本输入组件（已聚焦时不允许移除）。
  pub fn remove(&mut self, pool: &mut UiObjectPool, id: TextInputId) -> bool {
    if self.is_focused(pool, id) {
      return false;
    }
    let removed = pool.text_inputs.inputs.remove(&id).is_some();
    if removed {
      pool
        .events
        .retain(|event| event.text_input_id() != Some(id));
    }
    removed
  }

  /// 渲染文本输入组件到基础层，返回光标物理坐标。
  pub fn render(
    &self,
    pool: &mut UiObjectPool,
    id: TextInputId,
    params: &TextInputRenderParams,
    canvas: &mut CanvasService,
  ) -> Option<(u16, u16)> {
    self.render_target(pool, id, params, canvas, TextSurface::Base)
  }

  /// 渲染文本输入组件到指定切片，返回光标物理坐标。
  pub fn render_on(
    &self,
    pool: &mut UiObjectPool,
    id: TextInputId,
    slice: SliceId,
    params: &TextInputRenderParams,
    canvas: &mut CanvasService,
  ) -> Option<(u16, u16)> {
    self.render_target(pool, id, params, canvas, TextSurface::Slice(slice))
  }

  /// 渲染文本输入组件到宿主层。
  pub(crate) fn render_host(
    &self,
    pool: &mut UiObjectPool,
    id: TextInputId,
    params: &TextInputRenderParams,
    canvas: &mut CanvasService,
  ) -> Option<(u16, u16)> {
    self.render_target(pool, id, params, canvas, TextSurface::Host)
  }

  fn render_target(
    &self,
    pool: &mut UiObjectPool,
    id: TextInputId,
    params: &TextInputRenderParams,
    canvas: &mut CanvasService,
    surface: TextSurface,
  ) -> Option<(u16, u16)> {
    if params.rect.width == 0 || params.rect.height == 0 {
      if let Some(state) = pool.text_inputs.inputs.get_mut(&id) {
        state.hit = None;
      }
      return None;
    }
    let resolved = match surface {
      TextSurface::Base => canvas.base_hit_rect(params.rect),
      TextSurface::Slice(slice) => canvas.slice_hit_rect(slice, params.rect),
      TextSurface::Host => canvas.host_hit_rect(params.rect),
    };
    let Some((physical_rect, origin, surface_rank)) = resolved else {
      if let Some(state) = pool.text_inputs.inputs.get_mut(&id) {
        state.hit = None;
      }
      return None;
    };
    let mut params = params.clone();
    params.rect.width = physical_rect.width;
    params.rect.height = physical_rect.height;
    let order = pool.next_render_order();
    let active = self.is_focused(pool, id);
    let state = pool.text_inputs.inputs.get_mut(&id)?;
    fill_input_background(canvas, surface, &params);
    let cursor_visible = active
      && params.cursor_shape.unwrap_or_default() != TextInputCursorShape::None
      && (!params.cursor_blink || self.cursor_blink_visible());
    let result = match state.mode {
      TextInputMode::SingleLine => render_single_line(
        state,
        active,
        cursor_visible,
        &params,
        canvas,
        surface,
        order,
      ),
      TextInputMode::MultiLine => render_multi_line(
        state,
        active,
        cursor_visible,
        &params,
        canvas,
        surface,
        order,
      ),
    };
    if let Some(hit) = state.hit.as_mut() {
      hit.rect = physical_rect;
      hit.origin = origin;
      hit.surface_rank = surface_rank;
    }
    result.map(|(x, y)| (origin.0.saturating_add(x), origin.1.saturating_add(y)))
  }

  /// 聚焦指定文本输入组件，若之前有点击暂存则移动光标到该位置。
  pub fn focus(&mut self, pool: &mut UiObjectPool, id: TextInputId) -> bool {
    if self.active != TextInputActive::Inactive || !self.exists(pool, id) {
      return false;
    }
    self.active = TextInputActive::Focused(ActiveTextInput {
      pool_id: pool.id(),
      input_id: id,
    });
    let state = pool.text_inputs.inputs.get_mut(&id).unwrap();
    if let Some((cursor, line)) = state.pending_cursor.take() {
      state.buffer.move_to(cursor, false);
      state.visual_line = Some(line);
    }
    self.cursor_blink_started = Instant::now();
    pool.push_text_event(TextInputEvent::Focused { id });
    true
  }

  /// 取消当前焦点。
  pub fn blur(&mut self, pool: &mut UiObjectPool) -> bool {
    let TextInputActive::Focused(active) = self.active else {
      return false;
    };
    if active.pool_id != pool.id() || !self.exists(pool, active.input_id) {
      return false;
    }
    self.active = TextInputActive::Inactive;
    self.drag = None;
    pool.push_text_event(TextInputEvent::Blurred {
      id: active.input_id,
    });
    true
  }

  pub fn is_active(&self) -> bool {
    self.active != TextInputActive::Inactive
  }

  pub fn is_focused(&self, pool: &UiObjectPool, id: TextInputId) -> bool {
    self.active
      == TextInputActive::Focused(ActiveTextInput {
        pool_id: pool.id(),
        input_id: id,
      })
  }

  pub fn exists(&self, pool: &UiObjectPool, id: TextInputId) -> bool {
    pool.text_inputs.inputs.contains_key(&id)
  }

  pub fn get_text<'a>(&self, pool: &'a UiObjectPool, id: TextInputId) -> Option<&'a str> {
    pool
      .text_inputs
      .inputs
      .get(&id)
      .map(|state| state.buffer.text())
  }

  pub fn cursor(&self, pool: &UiObjectPool, id: TextInputId) -> Option<usize> {
    pool
      .text_inputs
      .inputs
      .get(&id)
      .map(|state| state.buffer.cursor())
  }

  pub fn selection(&self, pool: &UiObjectPool, id: TextInputId) -> Option<Range<usize>> {
    pool
      .text_inputs
      .inputs
      .get(&id)
      .and_then(|state| state.buffer.selection())
  }

  /// 设置输入框文本内容，触发 Changed 事件。
  pub fn set_text(
    &self,
    pool: &mut UiObjectPool,
    id: TextInputId,
    text: impl Into<String>,
  ) -> bool {
    let Some(state) = pool.text_inputs.inputs.get_mut(&id) else {
      return false;
    };
    if !state.buffer.set_text(text.into()) {
      return false;
    }
    state.visual_line = None;
    let value = state.buffer.text().to_string();
    pool.push_text_event(TextInputEvent::Changed { id, value });
    true
  }

  pub fn clear(&self, pool: &mut UiObjectPool, id: TextInputId) -> bool {
    self.set_text(pool, id, String::new())
  }

  /// 查找鼠标坐标下可命中的输入组件，返回（层级, 渲染顺序）用于排序。
  pub(crate) fn mouse_hit_order(
    &self,
    pool: &UiObjectPool,
    x: u16,
    y: u16,
  ) -> Option<(usize, u64)> {
    pool
      .text_inputs
      .inputs
      .values()
      .filter_map(|state| {
        state
          .mouse
          .then_some(state.hit?)
          .filter(|hit| hit.rect.contains(x, y))
      })
      .map(|hit| (hit.surface_rank, hit.order))
      .max()
  }

  /// 向当前聚焦的输入组件发送"外部按下"事件。
  pub(crate) fn push_pressed_outside(&self, pool: &mut UiObjectPool) {
    let TextInputActive::Focused(active) = self.active else {
      return;
    };
    if active.pool_id == pool.id()
      && pool
        .text_inputs
        .inputs
        .get(&active.input_id)
        .is_some_and(|state| state.mouse)
    {
      pool.push_text_event(TextInputEvent::PressedOutside {
        id: active.input_id,
      });
    }
  }

  /// 反激活指定对象池的命中区域和拖拽选区。
  pub(crate) fn deactivate_pool(&mut self, pool: &mut UiObjectPool) {
    pool.text_inputs.clear_hits();
    if self
      .drag
      .as_ref()
      .is_some_and(|drag| drag.active.pool_id == pool.id())
    {
      self.drag = None;
    }
  }
  fn cursor_blink_visible(&self) -> bool {
    (self.cursor_blink_started.elapsed().as_millis() / CURSOR_BLINK_INTERVAL.as_millis()) % 2 == 0
  }
}
