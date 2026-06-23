mod buffer;

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use self::buffer::TextBuffer;
use super::ui::UiObjectPool;
use super::{CanvasService, Rect, TerminalKeyCode, TerminalKeyEvent, TextColor, TextStyle};

const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextInputId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextInputMode {
  SingleLine,
  MultiLine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalAlign {
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

#[derive(Clone, Debug)]
pub struct TextInputOptions {
  pub initial_text: String,
  pub max_chars: Option<usize>,
  pub mode: TextInputMode,
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
}

struct TextInputState {
  buffer: TextBuffer,
  mode: TextInputMode,
}

pub(crate) struct TextInputObjects {
  next_input_id: u64,
  inputs: HashMap<TextInputId, TextInputState>,
  input_events: VecDeque<TextInputEvent>,
}

impl TextInputObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_input_id: 1,
      inputs: HashMap::new(),
      input_events: VecDeque::new(),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveTextInput {
  pool_id: u64,
  input_id: TextInputId,
}

pub struct TextInputService {
  active: Option<ActiveTextInput>,
  cursor_blink_started: Instant,
}

impl TextInputService {
  pub fn new() -> Self {
    Self {
      active: None,
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
        .text_inputs
        .input_events
        .retain(|event| event_id(event) != id);
    }
    removed
  }

  pub fn render(
    &self,
    pool: &UiObjectPool,
    id: TextInputId,
    params: &TextInputRenderParams,
    canvas: &mut CanvasService,
  ) -> Option<(u16, u16)> {
    if params.rect.width == 0 || params.rect.height == 0 {
      return None;
    }
    let state = pool.text_inputs.inputs.get(&id)?;
    let active = self.is_focused(pool, id);
    fill_input_background(canvas, params);
    let cursor_visible = active
      && params.cursor_shape.unwrap_or_default() != TextInputCursorShape::None
      && (!params.cursor_blink || self.cursor_blink_visible());
    match state.mode {
      TextInputMode::SingleLine => {
        render_single_line(state, active, cursor_visible, params, canvas)
      }
      TextInputMode::MultiLine => render_multi_line(state, active, cursor_visible, params, canvas),
    }
  }

  pub fn focus(&mut self, pool: &mut UiObjectPool, id: TextInputId) -> bool {
    if self.active.is_some() || !self.exists(pool, id) {
      return false;
    }
    self.active = Some(ActiveTextInput {
      pool_id: pool.id(),
      input_id: id,
    });
    self.cursor_blink_started = Instant::now();
    pool
      .text_inputs
      .input_events
      .push_back(TextInputEvent::Focused { id });
    true
  }

  pub fn blur(&mut self, pool: &mut UiObjectPool) -> bool {
    let Some(active) = self.active else {
      return false;
    };
    if active.pool_id != pool.id() || !self.exists(pool, active.input_id) {
      return false;
    }
    self.active = None;
    pool
      .text_inputs
      .input_events
      .push_back(TextInputEvent::Blurred {
        id: active.input_id,
      });
    true
  }

  pub fn is_active(&self) -> bool {
    self.active.is_some()
  }

  pub fn is_focused(&self, pool: &UiObjectPool, id: TextInputId) -> bool {
    self.active
      == Some(ActiveTextInput {
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
    pool
      .text_inputs
      .input_events
      .push_back(TextInputEvent::Changed {
        id,
        value: state.buffer.text().to_string(),
      });
    true
  }

  pub fn clear(&self, pool: &mut UiObjectPool, id: TextInputId) -> bool {
    self.set_text(pool, id, String::new())
  }

  pub fn take_events(&self, pool: &mut UiObjectPool, id: TextInputId) -> Vec<TextInputEvent> {
    let mut selected = Vec::new();
    let mut others = VecDeque::new();
    while let Some(event) = pool.text_inputs.input_events.pop_front() {
      if event_id(&event) == id {
        selected.push(event);
      } else {
        others.push_back(event);
      }
    }
    pool.text_inputs.input_events = others;
    selected
  }

  pub(crate) fn route_terminal_key(&mut self, pool: &mut UiObjectPool, key: TerminalKeyEvent) {
    let Some(active) = self.active else {
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

    let changed = match key.code {
      TerminalKeyCode::Char(ch) => state.buffer.insert_char(ch),
      TerminalKeyCode::Backspace => state.buffer.delete_prev(),
      TerminalKeyCode::Delete => state.buffer.delete_next(),
      TerminalKeyCode::Left => state.buffer.move_left(),
      TerminalKeyCode::Right => state.buffer.move_right(),
      TerminalKeyCode::Home => state.buffer.move_home(),
      TerminalKeyCode::End => state.buffer.move_end(),
      TerminalKeyCode::Enter if key.ctrl && state.mode == TextInputMode::MultiLine => {
        state.buffer.insert_newline()
      }
      TerminalKeyCode::Enter => {
        pool
          .text_inputs
          .input_events
          .push_back(TextInputEvent::Submit {
            id,
            value: state.buffer.text().to_string(),
          });
        return;
      }
      TerminalKeyCode::Esc => {
        pool
          .text_inputs
          .input_events
          .push_back(TextInputEvent::Cancel {
            id,
            value: state.buffer.text().to_string(),
          });
        return;
      }
    };

    if changed
      && matches!(
        key.code,
        TerminalKeyCode::Char(_)
          | TerminalKeyCode::Backspace
          | TerminalKeyCode::Delete
          | TerminalKeyCode::Enter
      )
    {
      pool
        .text_inputs
        .input_events
        .push_back(TextInputEvent::Changed {
          id,
          value: state.buffer.text().to_string(),
        });
    }
  }

  fn cursor_blink_visible(&self) -> bool {
    (self.cursor_blink_started.elapsed().as_millis() / CURSOR_BLINK_INTERVAL.as_millis()) % 2 == 0
  }
}

fn event_id(event: &TextInputEvent) -> TextInputId {
  match event {
    TextInputEvent::Focused { id }
    | TextInputEvent::Blurred { id }
    | TextInputEvent::Changed { id, .. }
    | TextInputEvent::Submit { id, .. }
    | TextInputEvent::Cancel { id, .. } => *id,
  }
}

fn render_single_line(
  state: &TextInputState,
  active: bool,
  cursor_visible: bool,
  params: &TextInputRenderParams,
  canvas: &mut CanvasService,
) -> Option<(u16, u16)> {
  let y = match params.vertical_align {
    VerticalAlign::Top => params.rect.y,
    VerticalAlign::Center => params.rect.y + (params.rect.height - 1) / 2,
    VerticalAlign::Bottom => params.rect.y + params.rect.height - 1,
  };
  if state.buffer.text().is_empty() {
    if !active {
      draw_prefix(
        canvas,
        params.rect.x,
        y,
        &params.placeholder,
        params.rect.width,
        input_placeholder_style(params),
      );
      return None;
    }
  }
  if !active {
    draw_prefix(
      canvas,
      params.rect.x,
      y,
      state.buffer.text(),
      params.rect.width,
      input_text_style(params),
    );
    return None;
  }

  let shape = params.cursor_shape.unwrap_or_default();
  let marker = cursor_marker(shape);
  let (before, current, after) = single_line_view(
    state.buffer.text(),
    state.buffer.cursor(),
    params.rect.width as usize,
    marker.map(UnicodeWidthStr::width).unwrap_or(0),
  );
  let cursor_x = params.rect.x + UnicodeWidthStr::width(before.as_str()) as u16;
  canvas.styled_text(params.rect.x, y, &before, input_text_style(params));
  let after_x = if let Some(current) = current {
    canvas.styled_text(
      cursor_x,
      y,
      &current,
      if cursor_visible {
        reversed_cursor_style(params)
      } else {
        input_text_style(params)
      },
    );
    cursor_x + UnicodeWidthStr::width(current.as_str()) as u16
  } else {
    if cursor_visible {
      if let Some(marker) = marker {
        canvas.styled_text(cursor_x, y, marker, input_cursor_style(params));
      }
    }
    cursor_x
  };
  canvas.styled_text(after_x, y, &after, input_text_style(params));
  cursor_in_canvas(canvas, cursor_x, y)
}

fn render_multi_line(
  state: &TextInputState,
  active: bool,
  cursor_visible: bool,
  params: &TextInputRenderParams,
  canvas: &mut CanvasService,
) -> Option<(u16, u16)> {
  if state.buffer.text().is_empty() && !active {
    draw_prefix(
      canvas,
      params.rect.x,
      params.rect.y,
      &params.placeholder,
      params.rect.width,
      input_placeholder_style(params),
    );
    return None;
  }

  let (tokens, cursor) = layout_multi_line(
    state.buffer.text(),
    state.buffer.cursor(),
    params.rect.width as usize,
    active,
    params.cursor_shape.unwrap_or_default(),
  );
  let first_line = cursor
    .map(|(line, _)| line.saturating_sub(params.rect.height as usize - 1))
    .unwrap_or(0);
  let last_line = first_line + params.rect.height as usize;
  for token in tokens
    .iter()
    .filter(|token| (first_line..last_line).contains(&token.line))
  {
    if token.cursor && token.marker && !cursor_visible {
      continue;
    }
    canvas.styled_text(
      params.rect.x + token.x as u16,
      params.rect.y + (token.line - first_line) as u16,
      &token.text,
      if token.cursor {
        if token.marker {
          input_cursor_style(params)
        } else if cursor_visible {
          reversed_cursor_style(params)
        } else {
          input_text_style(params)
        }
      } else {
        input_text_style(params)
      },
    );
  }

  let (line, x) = cursor?;
  cursor_in_canvas(
    canvas,
    params.rect.x + x as u16,
    params.rect.y + (line - first_line) as u16,
  )
}

struct VisualToken {
  text: String,
  line: usize,
  x: usize,
  cursor: bool,
  marker: bool,
}

fn layout_multi_line(
  text: &str,
  cursor: usize,
  width: usize,
  show_cursor: bool,
  shape: TextInputCursorShape,
) -> (Vec<VisualToken>, Option<(usize, usize)>) {
  let mut tokens = Vec::new();
  let mut line = 0;
  let mut x = 0;
  let mut cursor_position = None;

  for (start, grapheme) in text.grapheme_indices(true) {
    if show_cursor && start == cursor {
      if grapheme != "\n" {
        cursor_position = push_visual_token(
          &mut tokens,
          grapheme,
          UnicodeWidthStr::width(grapheme),
          true,
          false,
          width,
          &mut line,
          &mut x,
        );
        continue;
      }
      cursor_position = push_cursor_marker(&mut tokens, shape, width, &mut line, &mut x);
    }
    if grapheme == "\n" {
      line += 1;
      x = 0;
      continue;
    }
    push_visual_token(
      &mut tokens,
      grapheme,
      UnicodeWidthStr::width(grapheme),
      false,
      false,
      width,
      &mut line,
      &mut x,
    );
  }
  if show_cursor && cursor == text.len() {
    cursor_position = push_cursor_marker(&mut tokens, shape, width, &mut line, &mut x);
  }
  (tokens, cursor_position)
}

fn push_visual_token(
  tokens: &mut Vec<VisualToken>,
  text: &str,
  token_width: usize,
  cursor: bool,
  marker: bool,
  width: usize,
  line: &mut usize,
  x: &mut usize,
) -> Option<(usize, usize)> {
  if *x > 0 && *x + token_width > width {
    *line += 1;
    *x = 0;
  }
  if token_width > width {
    *line += 1;
    *x = 0;
    return None;
  }
  let position = (*line, *x);
  tokens.push(VisualToken {
    text: text.to_string(),
    line: *line,
    x: *x,
    cursor,
    marker,
  });
  *x += token_width;
  Some(position)
}

fn push_cursor_marker(
  tokens: &mut Vec<VisualToken>,
  shape: TextInputCursorShape,
  width: usize,
  line: &mut usize,
  x: &mut usize,
) -> Option<(usize, usize)> {
  if *x >= width {
    *line += 1;
    *x = 0;
  }
  let position = Some((*line, *x));
  if let Some(marker) = cursor_marker(shape) {
    push_visual_token(
      tokens,
      marker,
      UnicodeWidthStr::width(marker),
      true,
      true,
      width,
      line,
      x,
    );
  }
  position
}

fn cursor_in_canvas(canvas: &CanvasService, x: u16, y: u16) -> Option<(u16, u16)> {
  (x < canvas.width() && y < canvas.height()).then_some((x, y))
}

fn cursor_marker(shape: TextInputCursorShape) -> Option<&'static str> {
  match shape {
    TextInputCursorShape::Block => Some("█"),
    TextInputCursorShape::Underline => Some("_"),
    TextInputCursorShape::None => None,
    TextInputCursorShape::Line => Some("▏"),
  }
}

fn fill_input_background(canvas: &mut CanvasService, params: &TextInputRenderParams) {
  let line = " ".repeat(params.rect.width as usize);
  let style = TextStyle {
    background: params.bg.clone(),
    ..Default::default()
  };
  for offset in 0..params.rect.height {
    canvas.styled_text(
      params.rect.x,
      params.rect.y.saturating_add(offset),
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

fn reversed_cursor_style(params: &TextInputRenderParams) -> TextStyle {
  let mut style = input_text_style(params);
  style.reverse = !style.reverse;
  style
}

fn draw_prefix(
  canvas: &mut CanvasService,
  x: u16,
  y: u16,
  text: &str,
  width: u16,
  style: TextStyle,
) {
  let mut used = 0;
  let visible: String = text
    .graphemes(true)
    .take_while(|grapheme| {
      let next = used + UnicodeWidthStr::width(*grapheme);
      if next > width as usize {
        return false;
      }
      used = next;
      true
    })
    .collect();
  canvas.styled_text(x, y, &visible, style);
}

fn single_line_view(
  text: &str,
  cursor: usize,
  width: usize,
  end_cursor_width: usize,
) -> (String, Option<String>, String) {
  let graphemes: Vec<&str> = text.graphemes(true).collect();
  let cursor_index = graphemes
    .iter()
    .scan(0, |bytes, grapheme| {
      *bytes += grapheme.len();
      Some(*bytes)
    })
    .take_while(|end| *end <= cursor)
    .count();

  let mut start = cursor_index;
  let current = graphemes.get(cursor_index).copied();
  let mut used = current
    .map(UnicodeWidthStr::width)
    .unwrap_or(end_cursor_width);
  while start > 0 {
    let grapheme_width = UnicodeWidthStr::width(graphemes[start - 1]);
    if used + grapheme_width > width {
      break;
    }
    start -= 1;
    used += grapheme_width;
  }

  let mut end = cursor_index + usize::from(current.is_some());
  while end < graphemes.len() {
    let grapheme_width = UnicodeWidthStr::width(graphemes[end]);
    if used + grapheme_width > width {
      break;
    }
    used += grapheme_width;
    end += 1;
  }

  (
    graphemes[start..cursor_index].concat(),
    current.map(str::to_string),
    graphemes[cursor_index + usize::from(current.is_some())..end].concat(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  fn options(text: &str) -> TextInputOptions {
    TextInputOptions {
      initial_text: text.to_string(),
      max_chars: None,
      mode: TextInputMode::SingleLine,
    }
  }

  fn multiline_options(text: &str) -> TextInputOptions {
    TextInputOptions {
      initial_text: text.to_string(),
      max_chars: None,
      mode: TextInputMode::MultiLine,
    }
  }

  fn render_params(width: u16, placeholder: &str) -> TextInputRenderParams {
    TextInputRenderParams {
      rect: Rect {
        x: 0,
        y: 0,
        width,
        height: 1,
      },
      placeholder: placeholder.to_string(),
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

  #[test]
  fn pool_ids_are_unique_and_events_are_scoped() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let first = service.create(&mut pool, options(""));
    let second = service.create(&mut pool, options(""));
    assert_eq!((first, second), (TextInputId(1), TextInputId(2)));

    assert!(service.set_text(&mut pool, first, "a"));
    assert!(!service.set_text(&mut pool, first, "a"));
    assert!(service.set_text(&mut pool, second, "b"));
    assert!(service.remove(&mut pool, first));
    assert!(!service.exists(&pool, first));
    assert!(service.take_events(&mut pool, first).is_empty());
    assert_eq!(service.take_events(&mut pool, second).len(), 1);
    assert!(service.clear(&mut pool, second));
    assert_eq!(service.get_text(&pool, second), Some(""));
    assert!(matches!(
      service.take_events(&mut pool, second).as_slice(),
      [TextInputEvent::Changed { value, .. }] if value.is_empty()
    ));
  }

  #[test]
  fn focus_requires_valid_inactive_input() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let id = service.create(&mut pool, options(""));

    assert!(!service.focus(&mut pool, TextInputId(99)));
    assert!(service.focus(&mut pool, id));
    assert!(!service.focus(&mut pool, id));
    assert!(service.is_focused(&pool, id));
    assert!(service.blur(&mut pool));
    assert!(!service.blur(&mut pool));
    assert_eq!(
      service.take_events(&mut pool, id),
      vec![
        TextInputEvent::Focused { id },
        TextInputEvent::Blurred { id }
      ]
    );
  }

  #[test]
  fn service_edits_and_emits_only_real_changes() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let id = service.create(&mut pool, options(""));
    service.focus(&mut pool, id);
    service.take_events(&mut pool, id);

    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Char('我'),
        ctrl: false,
      },
    );
    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Left,
        ctrl: false,
      },
    );
    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Backspace,
        ctrl: false,
      },
    );
    assert_eq!(service.get_text(&pool, id), Some("我"));
    assert_eq!(service.take_events(&mut pool, id).len(), 1);

    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Enter,
        ctrl: false,
      },
    );
    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Esc,
        ctrl: false,
      },
    );
    let events = service.take_events(&mut pool, id);
    assert!(matches!(events[0], TextInputEvent::Submit { .. }));
    assert!(matches!(events[1], TextInputEvent::Cancel { .. }));
    assert!(service.is_active());
  }

  #[test]
  fn inactive_service_ignores_terminal_key() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let id = service.create(&mut pool, options(""));
    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Char('a'),
        ctrl: false,
      },
    );
    assert_eq!(service.get_text(&pool, id), Some(""));
  }

  #[test]
  fn draw_input_handles_placeholder_cursor_scroll_and_raw_prefix() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let empty = service.create(&mut pool, options(""));
    let raw = service.create(&mut pool, options("f%abc"));
    let cjk = service.create(&mut pool, options("你好abc"));
    let mut canvas = CanvasService::new();

    assert_eq!(
      service.render(&pool, empty, &render_params(5, "hint"), &mut canvas),
      None
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "h");
    canvas.clear();
    assert!(service.focus(&mut pool, empty));
    assert_eq!(
      service.render(&pool, empty, &render_params(5, "hint"), &mut canvas),
      Some((0, 0))
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "█");

    canvas.clear();
    assert!(service.blur(&mut pool));
    assert_eq!(
      service.render(&pool, raw, &render_params(6, ""), &mut canvas),
      None
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "f");
    assert_eq!(canvas.cell_at(1, 0).unwrap().text, "%");

    canvas.clear();
    assert!(service.focus(&mut pool, cjk));
    assert_eq!(
      service.render(&pool, cjk, &render_params(4, ""), &mut canvas),
      Some((3, 0))
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "a");
    assert_eq!(canvas.cell_at(3, 0).unwrap().text, "█");
  }

  #[test]
  fn fg_bg_fill_rect_and_reverse_uses_final_colors() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let text = service.create(&mut pool, options("f%a"));
    let empty = service.create(&mut pool, options(""));
    let mut canvas = CanvasService::new();
    let fg = Some(TextColor::Rgb { r: 1, g: 2, b: 3 });
    let bg = Some(TextColor::Rgb { r: 4, g: 5, b: 6 });
    let placeholder_fg = Some(TextColor::Rgb { r: 7, g: 8, b: 9 });
    let mut params = render_params(4, "f%hint");
    params.rect = Rect {
      x: 1,
      y: 2,
      width: 4,
      height: 3,
    };
    params.fg = fg.clone();
    params.bg = bg.clone();
    params.placeholder_fg = placeholder_fg.clone();
    params.text_style.foreground = Some(TextColor::Rgb {
      r: 99,
      g: 99,
      b: 99,
    });

    service.render(&pool, text, &params, &mut canvas);
    assert_eq!(canvas.cell_at(1, 2).unwrap().text, "f");
    assert_eq!(canvas.cell_at(2, 2).unwrap().text, "%");
    assert_eq!(
      canvas.cell_at(1, 2).unwrap().style.foreground.as_ref(),
      fg.as_ref()
    );
    assert_eq!(
      canvas.cell_at(1, 2).unwrap().style.background.as_ref(),
      bg.as_ref()
    );
    assert_eq!(
      canvas.cell_at(4, 4).unwrap().style.background.as_ref(),
      bg.as_ref()
    );
    canvas.clear();

    service.render(&pool, empty, &params, &mut canvas);
    assert_eq!(canvas.cell_at(1, 2).unwrap().text, "f");
    assert_eq!(canvas.cell_at(2, 2).unwrap().text, "%");
    assert_eq!(
      canvas.cell_at(1, 2).unwrap().style.foreground.as_ref(),
      placeholder_fg.as_ref()
    );
    assert_eq!(
      canvas.cell_at(1, 2).unwrap().style.background.as_ref(),
      bg.as_ref()
    );
    canvas.clear();

    assert!(service.focus(&mut pool, text));
    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Home,
        ctrl: false,
      },
    );
    service.render(&pool, text, &params, &mut canvas);
    let cursor = canvas.cell_at(1, 2).unwrap();
    assert!(cursor.style.reverse);
    assert_eq!(cursor.style.foreground.as_ref(), fg.as_ref());
    assert_eq!(cursor.style.background.as_ref(), bg.as_ref());
  }

  #[test]
  fn identical_ids_in_different_pools_do_not_share_focus() {
    let mut first_pool = UiObjectPool::new();
    let mut second_pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let first = service.create(&mut first_pool, options("first"));
    let second = service.create(&mut second_pool, options("second"));

    assert_eq!((first, second), (TextInputId(1), TextInputId(1)));
    assert!(service.focus(&mut first_pool, first));
    assert!(!service.blur(&mut second_pool));
    assert!(service.remove(&mut second_pool, second));
    assert!(!service.remove(&mut first_pool, first));
    assert!(service.blur(&mut first_pool));
    assert!(service.remove(&mut first_pool, first));
  }

  #[test]
  fn ctrl_enter_only_inserts_newline_in_multiline_mode() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let multiline = service.create(&mut pool, multiline_options("a"));
    let single = service.create(&mut pool, options("a"));

    assert!(service.focus(&mut pool, multiline));
    service.take_events(&mut pool, multiline);
    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Enter,
        ctrl: true,
      },
    );
    assert_eq!(service.get_text(&pool, multiline), Some("a\n"));
    assert!(matches!(
      service.take_events(&mut pool, multiline).as_slice(),
      [TextInputEvent::Changed { value, .. }] if value == "a\n"
    ));
    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Enter,
        ctrl: false,
      },
    );
    assert!(matches!(
      service.take_events(&mut pool, multiline).as_slice(),
      [TextInputEvent::Submit { value, .. }] if value == "a\n"
    ));

    assert!(service.blur(&mut pool));
    assert!(service.focus(&mut pool, single));
    service.take_events(&mut pool, single);
    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Enter,
        ctrl: true,
      },
    );
    assert_eq!(service.get_text(&pool, single), Some("a"));
    assert!(matches!(
      service.take_events(&mut pool, single).as_slice(),
      [TextInputEvent::Submit { value, .. }] if value == "a"
    ));
  }

  #[test]
  fn single_line_respects_vertical_alignment() {
    let mut pool = UiObjectPool::new();
    let service = TextInputService::new();
    let id = service.create(&mut pool, options("a"));
    let mut canvas = CanvasService::new();
    let mut params = render_params(3, "");
    params.rect.y = 2;
    params.rect.height = 3;

    service.render(&pool, id, &params, &mut canvas);
    assert_eq!(canvas.cell_at(0, 2).unwrap().text, "a");
    canvas.clear();

    params.vertical_align = VerticalAlign::Center;
    service.render(&pool, id, &params, &mut canvas);
    assert_eq!(canvas.cell_at(0, 3).unwrap().text, "a");
    canvas.clear();

    params.vertical_align = VerticalAlign::Bottom;
    service.render(&pool, id, &params, &mut canvas);
    assert_eq!(canvas.cell_at(0, 4).unwrap().text, "a");
  }

  #[test]
  fn multiline_wraps_by_cell_width_and_scrolls_to_cursor() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let id = service.create(&mut pool, multiline_options("abcdefghi"));
    let mut canvas = CanvasService::new();
    let mut params = render_params(3, "");
    params.rect.height = 2;

    service.render(&pool, id, &params, &mut canvas);
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "a");
    assert_eq!(canvas.cell_at(0, 1).unwrap().text, "d");
    canvas.clear();

    assert!(service.focus(&mut pool, id));
    assert_eq!(
      service.render(&pool, id, &params, &mut canvas),
      Some((0, 1))
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "g");
    assert_eq!(canvas.cell_at(0, 1).unwrap().text, "█");

    let (tokens, _) = layout_multi_line("a我b", 0, 3, false, TextInputCursorShape::Block);
    assert_eq!((tokens[0].line, tokens[1].line, tokens[2].line), (0, 0, 1));
  }

  #[test]
  fn cursor_reverses_complete_wide_grapheme_without_shifting_text() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let id = service.create(&mut pool, options("我爱你"));
    let mut canvas = CanvasService::new();
    assert!(service.focus(&mut pool, id));
    for _ in 0..2 {
      service.route_terminal_key(
        &mut pool,
        TerminalKeyEvent {
          code: TerminalKeyCode::Left,
          ctrl: false,
        },
      );
    }

    assert_eq!(
      service.render(&pool, id, &render_params(15, ""), &mut canvas),
      Some((2, 0))
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "我");
    assert_eq!(canvas.cell_at(2, 0).unwrap().text, "爱");
    assert!(canvas.cell_at(2, 0).unwrap().style.reverse);
    assert_eq!(canvas.cell_at(4, 0).unwrap().text, "你");

    let (tokens, cursor) =
      layout_multi_line("我爱你", "我".len(), 15, true, TextInputCursorShape::Block);
    assert_eq!(cursor, Some((0, 2)));
    assert_eq!((tokens[1].text.as_str(), tokens[1].x), ("爱", 2));
    assert!(tokens[1].cursor && !tokens[1].marker);
    assert_eq!((tokens[2].text.as_str(), tokens[2].x), ("你", 4));
  }

  #[test]
  fn optional_cursor_shape_only_changes_end_marker() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let id = service.create(&mut pool, options(""));
    let mut canvas = CanvasService::new();
    let mut params = render_params(5, "");
    assert!(service.focus(&mut pool, id));

    for (shape, marker) in [
      (None, "█"),
      (Some(TextInputCursorShape::Underline), "_"),
      (Some(TextInputCursorShape::Line), "▏"),
    ] {
      params.cursor_shape = shape;
      service.cursor_blink_started = Instant::now();
      service.render(&pool, id, &params, &mut canvas);
      assert_eq!(canvas.cell_at(0, 0).unwrap().text, marker);
      canvas.clear();
    }

    params.cursor_shape = Some(TextInputCursorShape::None);
    assert_eq!(
      service.render(&pool, id, &params, &mut canvas),
      Some((0, 0))
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, " ");
  }

  #[test]
  fn cursor_blinks_and_key_input_resets_visibility() {
    let mut pool = UiObjectPool::new();
    let mut service = TextInputService::new();
    let id = service.create(&mut pool, options(""));
    let mut canvas = CanvasService::new();
    let mut params = render_params(5, "");
    assert!(TextInputRenderParams::default().cursor_blink);
    assert!(service.focus(&mut pool, id));

    service.cursor_blink_started = Instant::now() - Duration::from_millis(600);
    assert_eq!(
      service.render(&pool, id, &params, &mut canvas),
      Some((0, 0))
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, " ");
    canvas.clear();

    params.cursor_blink = false;
    assert_eq!(
      service.render(&pool, id, &params, &mut canvas),
      Some((0, 0))
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "█");
    canvas.clear();
    params.cursor_blink = true;

    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Char('a'),
        ctrl: false,
      },
    );
    assert_eq!(
      service.render(&pool, id, &params, &mut canvas),
      Some((1, 0))
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "a");
    assert_eq!(canvas.cell_at(1, 0).unwrap().text, "█");
  }
}
