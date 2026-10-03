//! Service support for the widget service.

use std::ops::Range;
use std::time::{Duration, Instant};

use super::buffer::TextBuffer;
use super::render::{fill_input_background, render_multi_line, render_single_line};
use super::state::{ActiveTextInput, DragSelection, TextInputActive, TextInputState};
use super::types::{
  TextInputCursorShape, TextInputEvent, TextInputId, TextInputMode, TextInputOptions,
  TextInputRenderParams, TextSurface,
};
use crate::SliceId;
use crate::UiObjectPool;
use tg_service_canvas::CanvasService;

const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

/// The public entry point for text input operations.
///
/// # Fields
///
/// * `active` - The active.
/// * `drag` - The drag.
/// * `cursor_blink_started` - The cursor blink started.
pub struct TextInputService {
  /// The active.
  pub(super) active: TextInputActive,
  /// The drag.
  pub(super) drag: Option<DragSelection>,
  /// The cursor blink started.
  pub(super) cursor_blink_started: Instant,
}

impl TextInputService {
  /// Create a text input service with its initial state.
  pub fn new() -> Self {
    Self {
      active: TextInputActive::Inactive,
      drag: None,
      cursor_blink_started: Instant::now(),
    }
  }

  /// Create an owned text input object and return its identity.
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

  /// Remove the identified widget object and release its owned state.
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

  /// Render the text input service into its requested terminal-cell surface.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `params` - The formatting or rendering parameters.
  /// * `canvas` - The clipped canvas used for drawing.
  pub fn render(
    &self,
    pool: &mut UiObjectPool,
    id: TextInputId,
    params: &TextInputRenderParams,
    canvas: &mut CanvasService,
  ) -> Option<(u16, u16)> {
    self.render_target(pool, id, params, canvas, TextSurface::Base)
  }

  /// Render the component into the identified clipped slice and update its interaction geometry.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `slice` - The slice.
  /// * `params` - The formatting or rendering parameters.
  /// * `canvas` - The clipped canvas used for drawing.
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

  /// Render the component into the physical host surface and update its interaction geometry.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `params` - The formatting or rendering parameters.
  /// * `canvas` - The clipped canvas used for drawing.
  pub fn render_host(
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
    result.and_then(|(x, y)| {
      Some((
        u16::try_from(origin.0.saturating_add(i32::from(x))).ok()?,
        u16::try_from(origin.1.saturating_add(i32::from(y))).ok()?,
      ))
    })
  }

  /// Focus the requested input and apply a previously queued pointer cursor position when
  /// present.
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

  /// Release the focused text input and clear its active interaction state.
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

  /// Report whether this text input service is active.
  pub fn is_active(&self) -> bool {
    self.active != TextInputActive::Inactive
  }

  /// Report whether the addressed object is focused.
  pub fn is_focused(&self, pool: &UiObjectPool, id: TextInputId) -> bool {
    self.active
      == TextInputActive::Focused(ActiveTextInput {
        pool_id: pool.id(),
        input_id: id,
      })
  }

  /// Report whether the identified widget object is still present.
  pub fn exists(&self, pool: &UiObjectPool, id: TextInputId) -> bool {
    pool.text_inputs.inputs.contains_key(&id)
  }

  /// Return the current text of the identified input when the object is still live.
  pub fn get_text<'a>(&self, pool: &'a UiObjectPool, id: TextInputId) -> Option<&'a str> {
    pool
      .text_inputs
      .inputs
      .get(&id)
      .map(|state| state.buffer.text())
  }

  /// Return the cursor for the addressed object when it is available.
  pub fn cursor(&self, pool: &UiObjectPool, id: TextInputId) -> Option<usize> {
    pool
      .text_inputs
      .inputs
      .get(&id)
      .map(|state| state.buffer.cursor())
  }

  /// Return the selection for the addressed object when it is available.
  pub fn selection(&self, pool: &UiObjectPool, id: TextInputId) -> Option<Range<usize>> {
    pool
      .text_inputs
      .inputs
      .get(&id)
      .and_then(|state| state.buffer.selection())
  }

  /// Update the text used by this text input service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `text` - The text to process or display.
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

  /// Replace the identified input text with an empty string and queue its change event when
  /// needed.
  pub fn clear(&self, pool: &mut UiObjectPool, id: TextInputId) -> bool {
    self.set_text(pool, id, String::new())
  }

  /// Return the layer and drawing order of the top input hit at the supplied pointer position.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `x` - The horizontal coordinate in terminal cells.
  /// * `y` - The vertical coordinate in terminal cells.
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

  /// Queue an outside-press event for the currently focused input.
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

  /// Clear hit regions and drag state for a UI pool that is no longer interactive.
  pub fn deactivate_pool(&mut self, pool: &mut UiObjectPool) {
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
    (self.cursor_blink_started.elapsed().as_millis() / CURSOR_BLINK_INTERVAL.as_millis())
      .is_multiple_of(2)
  }
}

impl Default for TextInputService {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tg_service_layout::{LayoutService, Rect};

  fn test_canvas() -> (CanvasService, LayoutService) {
    let mut layout = LayoutService::new();
    layout.resize_physical(80, 24);
    layout.set_developer_viewport(Rect {
      x: 4,
      y: 3,
      width: 40,
      height: 12,
    });

    let mut canvas = CanvasService::new();
    canvas.resize(80, 24);
    (canvas, layout)
  }

  #[test]
  fn base_cursor_uses_the_clipped_viewport_origin_and_wide_cell_width() {
    let (mut canvas, layout) = test_canvas();
    let mut pool = UiObjectPool::new();
    pool.prepare_canvas(&mut canvas, &layout);
    let mut service = TextInputService::new();
    let input = service.create(
      &mut pool,
      TextInputOptions {
        initial_text: "A界".to_string(),
        ..Default::default()
      },
    );
    assert!(service.focus(&mut pool, input));

    let cursor = service.render(
      &mut pool,
      input,
      &TextInputRenderParams {
        rect: Rect {
          x: 2,
          y: 1,
          width: 10,
          height: 1,
        },
        cursor_blink: false,
        ..Default::default()
      },
      &mut canvas,
    );

    assert_eq!(cursor, Some((9, 4)));
  }

  #[test]
  fn multiline_cursor_tracks_the_visible_scrolled_line() {
    let (mut canvas, layout) = test_canvas();
    let mut pool = UiObjectPool::new();
    pool.prepare_canvas(&mut canvas, &layout);
    let mut service = TextInputService::new();
    let input = service.create(
      &mut pool,
      TextInputOptions {
        initial_text: "a\nb\nc".to_string(),
        mode: TextInputMode::MultiLine,
        ..Default::default()
      },
    );
    assert!(service.focus(&mut pool, input));

    let cursor = service.render(
      &mut pool,
      input,
      &TextInputRenderParams {
        rect: Rect {
          x: 3,
          y: 4,
          width: 5,
          height: 2,
        },
        cursor_blink: false,
        ..Default::default()
      },
      &mut canvas,
    );

    assert_eq!(cursor, Some((8, 8)));
  }

  #[test]
  fn host_cursor_stays_in_physical_coordinates_and_zero_rect_hides_it() {
    let (mut canvas, layout) = test_canvas();
    let mut pool = UiObjectPool::new();
    pool.prepare_canvas(&mut canvas, &layout);
    let mut service = TextInputService::new();
    let input = service.create(
      &mut pool,
      TextInputOptions {
        initial_text: "xy".to_string(),
        ..Default::default()
      },
    );
    assert!(service.focus(&mut pool, input));

    let host_cursor = service.render_host(
      &mut pool,
      input,
      &TextInputRenderParams {
        rect: Rect {
          x: 6,
          y: 2,
          width: 8,
          height: 1,
        },
        cursor_blink: false,
        ..Default::default()
      },
      &mut canvas,
    );
    let hidden_cursor = service.render_host(
      &mut pool,
      input,
      &TextInputRenderParams {
        rect: Rect {
          x: 6,
          y: 2,
          width: 0,
          height: 1,
        },
        ..Default::default()
      },
      &mut canvas,
    );

    assert_eq!(host_cursor, Some((8, 2)));
    assert_eq!(hidden_cursor, None);
  }
}
