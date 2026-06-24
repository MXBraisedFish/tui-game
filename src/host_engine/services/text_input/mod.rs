mod buffer;

use std::collections::HashMap;
use std::ops::Range;
use std::time::{Duration, Instant};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use self::buffer::TextBuffer;
use super::ui::{UiComponentEvent, UiObjectPool};
use super::{
  CanvasService, ClipboardService, MouseButton, MouseEvent, MouseEventKind, Rect, SliceId,
  TerminalKeyCode, TerminalKeyEvent, TextColor, TextStyle,
};

const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);
const DRAG_SCROLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextInputId(pub u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextInputMode {
  #[default]
  SingleLine,
  MultiLine,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VerticalAlign {
  #[default]
  Top,
  Center,
  Bottom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextInputCursorShape {
  #[default]
  Block,
  Underline,
  None,
  Line,
}

#[derive(Clone, Debug, Default)]
pub struct TextInputOptions {
  pub initial_text: String,
  pub max_chars: Option<usize>,
  pub mode: TextInputMode,
  pub mouse: bool,
}

#[derive(Clone, Debug)]
pub struct TextInputRenderParams {
  pub rect: Rect,
  pub placeholder: String,
  pub fg: Option<TextColor>,
  pub bg: Option<TextColor>,
  pub placeholder_fg: Option<TextColor>,
  pub text_style: TextStyle,
  pub placeholder_style: TextStyle,
  pub cursor_style: TextStyle,
  pub cursor_shape: Option<TextInputCursorShape>,
  pub cursor_blink: bool,
  pub vertical_align: VerticalAlign,
}

impl Default for TextInputRenderParams {
  fn default() -> Self {
    Self {
      rect: Rect::default(),
      placeholder: String::new(),
      fg: None,
      bg: None,
      placeholder_fg: None,
      text_style: TextStyle::default(),
      placeholder_style: TextStyle::default(),
      cursor_style: TextStyle::default(),
      cursor_shape: None,
      cursor_blink: true,
      vertical_align: VerticalAlign::Top,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextInputEvent {
  Focused { id: TextInputId },
  Blurred { id: TextInputId },
  Changed { id: TextInputId, value: String },
  Submit { id: TextInputId, value: String },
  Cancel { id: TextInputId, value: String },
  Pressed { id: TextInputId },
  PressedOutside { id: TextInputId },
}

#[derive(Clone, Copy)]
struct HitSnapshot {
  rect: Rect,
  origin: (u16, u16),
  surface_rank: usize,
  width: usize,
  first_line: usize,
  single_start: usize,
  order: u64,
}

#[derive(Clone, Copy)]
enum TextSurface {
  Base,
  Slice(SliceId),
  Host,
}

struct TextInputState {
  buffer: TextBuffer,
  mode: TextInputMode,
  mouse: bool,
  hit: Option<HitSnapshot>,
  pending_cursor: Option<(usize, usize)>,
  visual_line: Option<usize>,
}

pub(crate) struct TextInputObjects {
  next_input_id: u64,
  inputs: HashMap<TextInputId, TextInputState>,
}

impl TextInputObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_input_id: 1,
      inputs: HashMap::new(),
    }
  }

  pub(crate) fn clear_hits(&mut self) {
    for state in self.inputs.values_mut() {
      state.hit = None;
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveTextInput {
  pool_id: u64,
  input_id: TextInputId,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum TextInputActive {
  #[default]
  Inactive,
  Focused(ActiveTextInput),
}

struct DragSelection {
  active: ActiveTextInput,
  last_scroll: Instant,
}

pub struct TextInputService {
  active: TextInputActive,
  drag: Option<DragSelection>,
  cursor_blink_started: Instant,
}

impl TextInputService {
  pub fn new() -> Self {
    Self {
      active: TextInputActive::Inactive,
      drag: None,
      cursor_blink_started: Instant::now(),
    }
  }

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

  pub fn render(
    &self,
    pool: &mut UiObjectPool,
    id: TextInputId,
    params: &TextInputRenderParams,
    canvas: &mut CanvasService,
  ) -> Option<(u16, u16)> {
    self.render_target(pool, id, params, canvas, TextSurface::Base)
  }

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

  pub(crate) fn route_terminal_key(
    &mut self,
    pool: &mut UiObjectPool,
    clipboard: &mut ClipboardService,
    key: TerminalKeyEvent,
  ) {
    let TextInputActive::Focused(active) = self.active else {
      return;
    };
    if active.pool_id != pool.id() {
      return;
    }
    self.cursor_blink_started = Instant::now();
    let id = active.input_id;
    let Some(state) = pool.text_inputs.inputs.get_mut(&id) else {
      return;
    };
    let width = state.hit.map(|hit| hit.width).unwrap_or(1).max(1);
    let mut changed = false;

    match key.code {
      TerminalKeyCode::Char(ch) if key.ctrl => match ch.to_ascii_lowercase() {
        'a' => {
          state.buffer.select_all();
        }
        'c' => {
          if let Some(text) = state.buffer.selected_text() {
            clipboard.write_text(text);
          }
        }
        'x' => {
          if let Some(text) = state.buffer.selected_text().map(str::to_string) {
            if clipboard.write_text(&text) {
              changed = state.buffer.delete_selection();
            }
          }
        }
        'v' => {
          if let Some(text) = clipboard.read_text() {
            changed = state.buffer.insert_text(&text);
          }
        }
        _ => {}
      },
      TerminalKeyCode::Char(ch) => changed = state.buffer.insert_char(ch),
      TerminalKeyCode::Backspace => changed = state.buffer.delete_prev(),
      TerminalKeyCode::Delete => changed = state.buffer.delete_next(),
      TerminalKeyCode::Left => {
        state.buffer.move_left_select(key.shift, key.ctrl);
        state.visual_line = None;
      }
      TerminalKeyCode::Right => {
        state.buffer.move_right_select(key.shift, key.ctrl);
        state.visual_line = None;
      }
      TerminalKeyCode::Up => move_vertical(state, width, -1, key.shift),
      TerminalKeyCode::Down => move_vertical(state, width, 1, key.shift),
      TerminalKeyCode::Home => move_line_edge(state, width, false, key.shift),
      TerminalKeyCode::End => move_line_edge(state, width, true, key.shift),
      TerminalKeyCode::Enter if key.ctrl && state.mode == TextInputMode::MultiLine => {
        changed = state.buffer.insert_newline()
      }
      TerminalKeyCode::Enter => {
        pool
          .events
          .push_back(UiComponentEvent::TextInput(TextInputEvent::Submit {
            id,
            value: state.buffer.text().to_string(),
          }));
        return;
      }
      TerminalKeyCode::Esc => {
        pool
          .events
          .push_back(UiComponentEvent::TextInput(TextInputEvent::Cancel {
            id,
            value: state.buffer.text().to_string(),
          }));
        return;
      }
    }
    if changed {
      state.visual_line = None;
      pool
        .events
        .push_back(UiComponentEvent::TextInput(TextInputEvent::Changed {
          id,
          value: state.buffer.text().to_string(),
        }));
    }
  }

  pub(crate) fn route_mouse_event(&mut self, pool: &mut UiObjectPool, event: MouseEvent) -> bool {
    if event.button != Some(MouseButton::Left) && event.kind != MouseEventKind::Hold {
      if event.kind == MouseEventKind::Release {
        self.drag = None;
      }
      return false;
    }
    let pool_id = pool.id();
    match event.kind {
      MouseEventKind::Press => {
        let hit_id = pool
          .text_inputs
          .inputs
          .iter()
          .filter_map(|(id, state)| state.mouse.then_some((*id, state.hit?)))
          .filter(|(_, hit)| hit.rect.contains(event.x, event.y))
          .max_by_key(|(_, hit)| hit.order)
          .map(|(id, _)| id);
        if let Some(id) = hit_id {
          let focused = self.is_focused(pool, id);
          let state = pool.text_inputs.inputs.get_mut(&id).unwrap();
          let (cursor, line) = cursor_from_point(state, event.x, event.y);
          state.pending_cursor = Some((cursor, line));
          pool
            .events
            .push_back(UiComponentEvent::TextInput(TextInputEvent::Pressed { id }));
          if focused {
            state.buffer.move_to(cursor, false);
            state.visual_line = Some(line);
            state.pending_cursor = None;
            self.drag = Some(DragSelection {
              active: ActiveTextInput {
                pool_id,
                input_id: id,
              },
              last_scroll: Instant::now(),
            });
          }
          self.cursor_blink_started = Instant::now();
          true
        } else if let TextInputActive::Focused(active) = self.active {
          if active.pool_id != pool_id {
            return false;
          }
          if pool
            .text_inputs
            .inputs
            .get(&active.input_id)
            .is_some_and(|state| state.mouse)
          {
            pool.events.push_back(UiComponentEvent::TextInput(
              TextInputEvent::PressedOutside {
                id: active.input_id,
              },
            ));
            true
          } else {
            false
          }
        } else {
          false
        }
      }
      MouseEventKind::Drag | MouseEventKind::Hold => self.drag_selection(pool, event),
      MouseEventKind::Release => {
        let consumed = self
          .drag
          .as_ref()
          .is_some_and(|drag| drag.active.pool_id == pool_id);
        self.drag = None;
        consumed
      }
      _ => false,
    }
  }

  fn drag_selection(&mut self, pool: &mut UiObjectPool, event: MouseEvent) -> bool {
    let Some(drag) = self.drag.as_mut() else {
      return false;
    };
    if drag.active.pool_id != pool.id() {
      return false;
    }
    let Some(state) = pool.text_inputs.inputs.get_mut(&drag.active.input_id) else {
      return false;
    };
    let Some(hit) = state.hit else { return false };
    if hit.rect.contains(event.x, event.y) {
      let (cursor, line) = cursor_from_point(state, event.x, event.y);
      state.buffer.move_to(cursor, true);
      state.buffer.set_preferred_column(None);
      state.visual_line = Some(line);
    } else if drag.last_scroll.elapsed() >= DRAG_SCROLL_INTERVAL {
      let layout = VisualLayout::new(state.buffer.text(), hit.width);
      if event.y < hit.rect.y {
        move_vertical(state, hit.width, -1, true);
      } else if event.y >= hit.rect.y.saturating_add(hit.rect.height) {
        move_vertical(state, hit.width, 1, true);
      } else if event.x < hit.rect.x {
        state.buffer.move_left_select(true, false);
      } else if event.x >= hit.rect.x.saturating_add(hit.rect.width) {
        state.buffer.move_right_select(true, false);
      }
      let (line, _) = layout.position(state.buffer.cursor(), state.visual_line);
      state.visual_line = Some(line);
      drag.last_scroll = Instant::now();
    }
    self.cursor_blink_started = Instant::now();
    true
  }

  fn cursor_blink_visible(&self) -> bool {
    (self.cursor_blink_started.elapsed().as_millis() / CURSOR_BLINK_INTERVAL.as_millis()) % 2 == 0
  }
}

#[derive(Clone)]
struct VisualGlyph {
  start: usize,
  end: usize,
  text: String,
  line: usize,
  x: usize,
  width: usize,
}

#[derive(Clone, Copy)]
struct VisualLine {
  start: usize,
  end: usize,
}

struct VisualLayout {
  glyphs: Vec<VisualGlyph>,
  lines: Vec<VisualLine>,
}

impl VisualLayout {
  fn new(text: &str, width: usize) -> Self {
    let width = width.max(1);
    let mut glyphs = Vec::new();
    let mut lines = Vec::new();
    let (mut line_start, mut line, mut x) = (0, 0, 0);
    for (start, grapheme) in text.grapheme_indices(true) {
      let end = start + grapheme.len();
      if grapheme == "\n" {
        lines.push(VisualLine {
          start: line_start,
          end: start,
        });
        line += 1;
        x = 0;
        line_start = end;
        continue;
      }
      let glyph_width = UnicodeWidthStr::width(grapheme);
      if x > 0 && x + glyph_width > width {
        lines.push(VisualLine {
          start: line_start,
          end: start,
        });
        line += 1;
        x = 0;
        line_start = start;
      }
      if glyph_width <= width {
        glyphs.push(VisualGlyph {
          start,
          end,
          text: grapheme.to_string(),
          line,
          x,
          width: glyph_width,
        });
        x += glyph_width;
      }
    }
    lines.push(VisualLine {
      start: line_start,
      end: text.len(),
    });
    Self { glyphs, lines }
  }

  fn position(&self, cursor: usize, hint: Option<usize>) -> (usize, usize) {
    let line = hint
      .filter(|line| {
        self
          .lines
          .get(*line)
          .is_some_and(|row| (row.start..=row.end).contains(&cursor))
      })
      .or_else(|| {
        self
          .lines
          .iter()
          .enumerate()
          .rev()
          .find(|(_, row)| (row.start..=row.end).contains(&cursor))
          .map(|(line, _)| line)
      })
      .unwrap_or(0);
    let x = self
      .glyphs
      .iter()
      .filter(|glyph| glyph.line == line && glyph.end <= cursor)
      .map(|glyph| glyph.width)
      .sum();
    (line, x)
  }

  fn boundary_at(&self, line: usize, x: usize) -> usize {
    let Some(row) = self.lines.get(line) else {
      return self.lines.last().map(|line| line.end).unwrap_or(0);
    };
    for glyph in self.glyphs.iter().filter(|glyph| glyph.line == line) {
      if x <= glyph.x {
        return glyph.start;
      }
      if x < glyph.x + glyph.width {
        return glyph.end;
      }
    }
    row.end
  }
}

fn move_vertical(state: &mut TextInputState, width: usize, delta: isize, extend: bool) {
  if !extend {
    if let Some(range) = state.buffer.selection() {
      state
        .buffer
        .move_to(if delta < 0 { range.start } else { range.end }, false);
      state.visual_line = None;
      return;
    }
  }
  if state.mode == TextInputMode::SingleLine {
    return;
  }
  let layout = VisualLayout::new(state.buffer.text(), width);
  let (line, x) = layout.position(state.buffer.cursor(), state.visual_line);
  let preferred = state.buffer.preferred_column().unwrap_or(x);
  let target =
    (line as isize + delta).clamp(0, layout.lines.len().saturating_sub(1) as isize) as usize;
  if target == line {
    return;
  }
  state
    .buffer
    .set_cursor(layout.boundary_at(target, preferred), extend);
  state.buffer.set_preferred_column(Some(preferred));
  state.visual_line = Some(target);
}

fn move_line_edge(state: &mut TextInputState, width: usize, end: bool, extend: bool) {
  let layout = VisualLayout::new(state.buffer.text(), width);
  let (line, _) = layout.position(state.buffer.cursor(), state.visual_line);
  let row = layout.lines[line];
  state
    .buffer
    .move_to(if end { row.end } else { row.start }, extend);
  state.visual_line = Some(line);
}

fn cursor_from_point(state: &TextInputState, x: u16, y: u16) -> (usize, usize) {
  let hit = state.hit.unwrap();
  let layout = VisualLayout::new(state.buffer.text(), hit.width);
  let line = if state.mode == TextInputMode::SingleLine {
    0
  } else {
    hit.first_line + y.saturating_sub(hit.rect.y) as usize
  }
  .min(layout.lines.len().saturating_sub(1));
  let local_x = x.saturating_sub(hit.rect.x) as usize;
  let cursor = if state.mode == TextInputMode::SingleLine {
    let start_x = layout.position(hit.single_start, Some(0)).1;
    layout.boundary_at(0, start_x + local_x)
  } else {
    layout.boundary_at(line, local_x)
  };
  (cursor, line)
}

fn render_single_line(
  state: &mut TextInputState,
  active: bool,
  cursor_visible: bool,
  params: &TextInputRenderParams,
  canvas: &mut CanvasService,
  surface: TextSurface,
  order: u64,
) -> Option<(u16, u16)> {
  let y = match params.vertical_align {
    VerticalAlign::Top => params.rect.y,
    VerticalAlign::Center => params.rect.y + (params.rect.height - 1) / 2,
    VerticalAlign::Bottom => params.rect.y + params.rect.height - 1,
  };
  if state.buffer.text().is_empty() {
    draw_placeholder(canvas, surface, y, params);
  }
  if state.buffer.text().is_empty() && !active {
    state.hit = Some(HitSnapshot {
      rect: params.rect,
      origin: (0, 0),
      surface_rank: 0,
      width: params.rect.width as usize,
      first_line: 0,
      single_start: 0,
      order,
    });
    return None;
  }
  let layout = VisualLayout::new(state.buffer.text(), usize::MAX / 2);
  let (_, cursor_x_full) = layout.position(state.buffer.cursor(), Some(0));
  let cursor_glyph = layout
    .glyphs
    .iter()
    .find(|glyph| glyph.start == state.buffer.cursor());
  let cursor_width = cursor_glyph.map(|glyph| glyph.width).unwrap_or_else(|| {
    cursor_marker(params.cursor_shape.unwrap_or_default())
      .map(UnicodeWidthStr::width)
      .unwrap_or(0)
  });
  let mut start = state.buffer.cursor();
  let mut used = cursor_width;
  for glyph in layout
    .glyphs
    .iter()
    .rev()
    .filter(|glyph| glyph.end <= state.buffer.cursor())
  {
    if used + glyph.width > params.rect.width as usize {
      break;
    }
    used += glyph.width;
    start = glyph.start;
  }
  let start_x = layout.position(start, Some(0)).1;
  let mut x = 0;
  let selection = active.then(|| state.buffer.selection()).flatten();
  for glyph in layout.glyphs.iter().filter(|glyph| glyph.end > start) {
    if x + glyph.width > params.rect.width as usize {
      break;
    }
    let at_cursor = active && glyph.start == state.buffer.cursor();
    let selected = selection
      .as_ref()
      .is_some_and(|range| range.start < glyph.end && glyph.start < range.end);
    let style = if (at_cursor && cursor_visible) || selected {
      reversed_text_style(params)
    } else {
      input_text_style(params)
    };
    draw_styled(
      canvas,
      surface,
      params.rect.x + x as u16,
      y,
      &glyph.text,
      style,
    );
    x += glyph.width;
  }
  let cursor_x = cursor_x_full.saturating_sub(start_x);
  if active && cursor_glyph.is_none() && cursor_visible {
    if let Some(marker) = cursor_marker(params.cursor_shape.unwrap_or_default()) {
      draw_styled(
        canvas,
        surface,
        params.rect.x + cursor_x as u16,
        y,
        marker,
        input_cursor_style(params),
      );
    }
  }
  state.hit = Some(HitSnapshot {
    rect: params.rect,
    origin: (0, 0),
    surface_rank: 0,
    width: params.rect.width as usize,
    first_line: 0,
    single_start: start,
    order,
  });
  active.then_some((params.rect.x + cursor_x as u16, y))
}

fn render_multi_line(
  state: &mut TextInputState,
  active: bool,
  cursor_visible: bool,
  params: &TextInputRenderParams,
  canvas: &mut CanvasService,
  surface: TextSurface,
  order: u64,
) -> Option<(u16, u16)> {
  if state.buffer.text().is_empty() {
    draw_placeholder(canvas, surface, params.rect.y, params);
  }
  if state.buffer.text().is_empty() && !active {
    state.hit = Some(HitSnapshot {
      rect: params.rect,
      origin: (0, 0),
      surface_rank: 0,
      width: params.rect.width as usize,
      first_line: 0,
      single_start: 0,
      order,
    });
    return None;
  }
  let layout = VisualLayout::new(state.buffer.text(), params.rect.width as usize);
  let (mut cursor_line, mut cursor_x) = layout.position(state.buffer.cursor(), state.visual_line);
  if active
    && !layout
      .glyphs
      .iter()
      .any(|glyph| glyph.start == state.buffer.cursor())
    && cursor_x >= params.rect.width as usize
  {
    cursor_line += 1;
    cursor_x = 0;
  }
  let first_line = if active {
    cursor_line.saturating_sub(params.rect.height as usize - 1)
  } else {
    0
  };
  let selection = active.then(|| state.buffer.selection()).flatten();
  for glyph in layout
    .glyphs
    .iter()
    .filter(|glyph| (first_line..first_line + params.rect.height as usize).contains(&glyph.line))
  {
    let at_cursor = active && glyph.start == state.buffer.cursor();
    let selected = selection
      .as_ref()
      .is_some_and(|range| range.start < glyph.end && glyph.start < range.end);
    let style = if (at_cursor && cursor_visible) || selected {
      reversed_text_style(params)
    } else {
      input_text_style(params)
    };
    draw_styled(
      canvas,
      surface,
      params.rect.x + glyph.x as u16,
      params.rect.y + (glyph.line - first_line) as u16,
      &glyph.text,
      style,
    );
  }
  if active
    && !layout
      .glyphs
      .iter()
      .any(|glyph| glyph.start == state.buffer.cursor())
    && cursor_visible
  {
    if let Some(marker) = cursor_marker(params.cursor_shape.unwrap_or_default()) {
      draw_styled(
        canvas,
        surface,
        params.rect.x + cursor_x as u16,
        params.rect.y + (cursor_line - first_line) as u16,
        marker,
        input_cursor_style(params),
      );
    }
  }
  state.hit = Some(HitSnapshot {
    rect: params.rect,
    origin: (0, 0),
    surface_rank: 0,
    width: params.rect.width as usize,
    first_line,
    single_start: 0,
    order,
  });
  active.then_some((
    params.rect.x + cursor_x as u16,
    params.rect.y + (cursor_line - first_line) as u16,
  ))
}

fn cursor_marker(shape: TextInputCursorShape) -> Option<&'static str> {
  match shape {
    TextInputCursorShape::Block => Some("█"),
    TextInputCursorShape::Underline => Some("_"),
    TextInputCursorShape::None => None,
    TextInputCursorShape::Line => Some("▏"),
  }
}

fn fill_input_background(
  canvas: &mut CanvasService,
  surface: TextSurface,
  params: &TextInputRenderParams,
) {
  let line = " ".repeat(params.rect.width as usize);
  let style = TextStyle {
    background: params.bg.clone(),
    ..Default::default()
  };
  for y in 0..params.rect.height {
    draw_styled(
      canvas,
      surface,
      params.rect.x,
      params.rect.y + y,
      &line,
      style.clone(),
    );
  }
}

fn input_text_style(params: &TextInputRenderParams) -> TextStyle {
  TextStyle {
    foreground: params.fg.clone(),
    background: params.bg.clone(),
    ..params.text_style.clone()
  }
}
fn input_placeholder_style(params: &TextInputRenderParams) -> TextStyle {
  TextStyle {
    foreground: params.placeholder_fg.clone(),
    background: params.bg.clone(),
    ..params.placeholder_style.clone()
  }
}
fn input_cursor_style(params: &TextInputRenderParams) -> TextStyle {
  TextStyle {
    background: params.bg.clone(),
    ..params.cursor_style.clone()
  }
}
fn reversed_text_style(params: &TextInputRenderParams) -> TextStyle {
  let mut style = input_text_style(params);
  style.reverse = !style.reverse;
  style
}

fn draw_prefix(
  canvas: &mut CanvasService,
  surface: TextSurface,
  x: u16,
  y: u16,
  text: &str,
  width: u16,
  style: TextStyle,
) {
  let mut used = 0;
  let text: String = text
    .graphemes(true)
    .take_while(|grapheme| {
      let next = used + UnicodeWidthStr::width(*grapheme);
      if next > width as usize {
        false
      } else {
        used = next;
        true
      }
    })
    .collect();
  draw_styled(canvas, surface, x, y, &text, style);
}

fn draw_placeholder(
  canvas: &mut CanvasService,
  surface: TextSurface,
  y: u16,
  params: &TextInputRenderParams,
) {
  draw_prefix(
    canvas,
    surface,
    params.rect.x.saturating_add(1),
    y,
    &params.placeholder,
    params.rect.width.saturating_sub(1),
    input_placeholder_style(params),
  );
}

fn draw_styled(
  canvas: &mut CanvasService,
  surface: TextSurface,
  x: u16,
  y: u16,
  text: &str,
  style: TextStyle,
) {
  match surface {
    TextSurface::Base => canvas.styled_text(x, y, text, style),
    TextSurface::Slice(id) => {
      canvas.styled_text_on(id, x, y, text, style);
    }
    TextSurface::Host => canvas.host_styled_text(x, y, text, style),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{
    LayoutService, SliceLength, SliceOptions, SliceRect, SliceService,
  };

  fn options(text: &str, mode: TextInputMode) -> TextInputOptions {
    TextInputOptions {
      initial_text: text.into(),
      mode,
      mouse: true,
      ..Default::default()
    }
  }
  fn params(width: u16, height: u16) -> TextInputRenderParams {
    TextInputRenderParams {
      rect: Rect {
        x: 0,
        y: 0,
        width,
        height,
      },
      cursor_blink: false,
      ..Default::default()
    }
  }
  fn key(code: TerminalKeyCode, ctrl: bool, shift: bool) -> TerminalKeyEvent {
    TerminalKeyEvent { code, ctrl, shift }
  }
  fn text_events(pool: &mut UiObjectPool) -> Vec<TextInputEvent> {
    std::iter::from_fn(|| pool.pop_event())
      .filter_map(|event| match event {
        super::super::UiEvent::TextInput(event) => Some(event),
        _ => None,
      })
      .collect()
  }

  #[test]
  fn selection_navigation_and_replacement_are_grapheme_safe() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let mut clipboard = ClipboardService::new();
    let id = service.create(&mut pool, options("我爱👨‍👩", TextInputMode::SingleLine));
    service.focus(&mut pool, id);
    service.route_terminal_key(
      &mut pool,
      &mut clipboard,
      key(TerminalKeyCode::Left, false, true),
    );
    service.route_terminal_key(
      &mut pool,
      &mut clipboard,
      key(TerminalKeyCode::Left, false, true),
    );
    service.route_terminal_key(
      &mut pool,
      &mut clipboard,
      key(TerminalKeyCode::Char('x'), false, false),
    );
    assert_eq!(service.get_text(&pool, id), Some("我x"));
  }

  #[test]
  fn cursor_and_selection_queries_return_grapheme_boundary_bytes() {
    let text = "a我e\u{301}👨‍👩";
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let mut clipboard = ClipboardService::new();
    let id = service.create(&mut pool, options(text, TextInputMode::SingleLine));
    assert_eq!(service.cursor(&pool, id), Some(text.len()));
    assert_eq!(service.selection(&pool, id), None);

    service.focus(&mut pool, id);
    service.route_terminal_key(
      &mut pool,
      &mut clipboard,
      key(TerminalKeyCode::Left, false, true),
    );
    assert_eq!(
      service.selection(&pool, id),
      Some("a我e\u{301}".len()..text.len())
    );
    service.route_terminal_key(
      &mut pool,
      &mut clipboard,
      key(TerminalKeyCode::Left, false, true),
    );
    let selection = service.selection(&pool, id).unwrap();
    assert_eq!(selection, "a我".len()..text.len());
    assert!(text.is_char_boundary(selection.start));
    assert!(text.is_char_boundary(selection.end));
    assert_eq!(service.cursor(&pool, TextInputId(99)), None);
  }

  #[test]
  fn placeholder_keeps_one_cell_for_cursor_while_focused() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let id = service.create(&mut pool, options("", TextInputMode::SingleLine));
    let mut canvas = CanvasService::new();
    let mut render_params = params(6, 1);
    render_params.placeholder = "hint".into();

    service.render(&mut pool, id, &render_params, &mut canvas);
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, " ");
    assert_eq!(canvas.cell_at(1, 0).unwrap().text, "h");

    service.focus(&mut pool, id);
    canvas.clear();
    service.render(&mut pool, id, &render_params, &mut canvas);
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "█");
    assert_eq!(canvas.cell_at(1, 0).unwrap().text, "h");
  }

  #[test]
  fn visual_line_navigation_and_edges_use_wrapped_rows() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let mut clipboard = ClipboardService::new();
    let id = service.create(&mut pool, options("abcdefghi", TextInputMode::MultiLine));
    let mut canvas = CanvasService::new();
    service.focus(&mut pool, id);
    service.render(&mut pool, id, &params(3, 2), &mut canvas);
    service.route_terminal_key(
      &mut pool,
      &mut clipboard,
      key(TerminalKeyCode::Up, false, false),
    );
    service.route_terminal_key(
      &mut pool,
      &mut clipboard,
      key(TerminalKeyCode::Home, false, false),
    );
    service.route_terminal_key(
      &mut pool,
      &mut clipboard,
      key(TerminalKeyCode::Char('X'), false, false),
    );
    assert_eq!(service.get_text(&pool, id), Some("abcXdefghi"));
  }

  #[test]
  fn selection_renders_reversed_and_inactive_hides_it() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let mut clipboard = ClipboardService::new();
    let id = service.create(&mut pool, options("我爱你", TextInputMode::SingleLine));
    let mut canvas = CanvasService::new();
    service.focus(&mut pool, id);
    service.route_terminal_key(
      &mut pool,
      &mut clipboard,
      key(TerminalKeyCode::Left, false, true),
    );
    service.render(&mut pool, id, &params(10, 1), &mut canvas);
    assert!(canvas.cell_at(4, 0).unwrap().style.reverse);
    service.blur(&mut pool);
    canvas.clear();
    service.render(&mut pool, id, &params(10, 1), &mut canvas);
    assert!(!canvas.cell_at(4, 0).unwrap().style.reverse);
  }

  #[test]
  fn mouse_click_is_deferred_until_focus_and_drag_selects() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let id = service.create(&mut pool, options("abc", TextInputMode::SingleLine));
    let mut canvas = CanvasService::new();
    service.render(&mut pool, id, &params(5, 1), &mut canvas);
    assert!(service.route_mouse_event(
      &mut pool,
      MouseEvent {
        kind: MouseEventKind::Press,
        button: Some(MouseButton::Left),
        scroll: None,
        x: 1,
        y: 0
      }
    ));
    assert_eq!(text_events(&mut pool), vec![TextInputEvent::Pressed { id }]);
    assert!(service.focus(&mut pool, id));
    assert!(service.route_mouse_event(
      &mut pool,
      MouseEvent {
        kind: MouseEventKind::Press,
        button: Some(MouseButton::Left),
        scroll: None,
        x: 0,
        y: 0
      }
    ));
    assert!(service.route_mouse_event(
      &mut pool,
      MouseEvent {
        kind: MouseEventKind::Drag,
        button: Some(MouseButton::Left),
        scroll: None,
        x: 2,
        y: 0
      }
    ));
    assert_eq!(
      pool.text_inputs.inputs[&id].buffer.selected_text(),
      Some("ab")
    );
  }

  #[test]
  fn slice_render_uses_physical_ime_anchor_and_mouse_coordinates() {
    let slices = SliceService::new();
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let id = service.create(&mut pool, options("a", TextInputMode::SingleLine));
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    layout.set_developer_viewport(Rect {
      x: 3,
      y: 2,
      width: 10,
      height: 6,
    });
    let slice = slices
      .create(
        &mut pool,
        SliceOptions {
          rect: SliceRect {
            x: 2,
            y: 1,
            width: SliceLength::Fixed(5),
            height: SliceLength::Fixed(3),
          },
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    service.focus(&mut pool, id);
    let mut render_params = params(4, 1);
    render_params.rect.x = 1;
    render_params.rect.y = 1;

    assert_eq!(
      service.render_on(&mut pool, id, slice, &render_params, &mut canvas),
      Some((7, 4))
    );
    assert!(service.route_mouse_event(
      &mut pool,
      MouseEvent {
        kind: MouseEventKind::Press,
        button: Some(MouseButton::Left),
        scroll: None,
        x: 6,
        y: 4,
      },
    ));
    assert_eq!(service.cursor(&pool, id), Some(0));
  }

  #[test]
  fn mouse_defaults_to_disabled() {
    assert!(!TextInputOptions::default().mouse);
    let mut pool = UiObjectPool::new();
    let service = TextInputService::new();
    let id = service.create(&mut pool, TextInputOptions::default());
    let mut canvas = CanvasService::new();
    service.render(&mut pool, id, &params(5, 1), &mut canvas);
    let mut service = service;
    assert!(!service.route_mouse_event(
      &mut pool,
      MouseEvent {
        kind: MouseEventKind::Press,
        button: Some(MouseButton::Left),
        scroll: None,
        x: 0,
        y: 0
      }
    ));
  }

  #[test]
  fn clipboard_shortcuts_copy_cut_and_paste_once() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let id = service.create(&mut pool, options("abc", TextInputMode::MultiLine));
    service.focus(&mut pool, id);
    text_events(&mut pool);

    let mut unavailable = ClipboardService::unavailable();
    service.route_terminal_key(
      &mut pool,
      &mut unavailable,
      key(TerminalKeyCode::Char('a'), true, false),
    );
    service.route_terminal_key(
      &mut pool,
      &mut unavailable,
      key(TerminalKeyCode::Char('x'), true, false),
    );
    assert_eq!(service.get_text(&pool, id), Some("abc"));

    let mut clipboard = ClipboardService::memory("");
    service.route_terminal_key(
      &mut pool,
      &mut clipboard,
      key(TerminalKeyCode::Char('c'), true, false),
    );
    assert_eq!(clipboard.read_text().as_deref(), Some("abc"));
    service.route_terminal_key(
      &mut pool,
      &mut clipboard,
      key(TerminalKeyCode::Char('x'), true, false),
    );
    assert_eq!(service.get_text(&pool, id), Some(""));
    assert_eq!(text_events(&mut pool).len(), 1);

    clipboard.write_text("我\r\n🌍");
    service.route_terminal_key(
      &mut pool,
      &mut clipboard,
      key(TerminalKeyCode::Char('v'), true, false),
    );
    assert_eq!(service.get_text(&pool, id), Some("我\n🌍"));
    assert_eq!(text_events(&mut pool).len(), 1);
  }

  #[test]
  fn public_lifecycle_and_outside_click_events_are_consistent() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let first = service.create(&mut pool, options("a", TextInputMode::SingleLine));
    let second = service.create(&mut pool, options("b", TextInputMode::SingleLine));
    assert_eq!((first, second), (TextInputId(1), TextInputId(2)));
    assert!(!service.set_text(&mut pool, first, "a"));
    assert!(service.focus(&mut pool, first));
    assert!(!service.remove(&mut pool, first));

    let mut canvas = CanvasService::new();
    service.render(&mut pool, first, &params(5, 1), &mut canvas);
    assert!(service.route_mouse_event(
      &mut pool,
      MouseEvent {
        kind: MouseEventKind::Press,
        button: Some(MouseButton::Left),
        scroll: None,
        x: 9,
        y: 0,
      },
    ));
    assert!(text_events(&mut pool).contains(&TextInputEvent::PressedOutside { id: first }));
    assert!(service.blur(&mut pool));
    assert!(service.remove(&mut pool, first));
    assert!(!service.exists(&pool, first));
  }
}
