mod buffer;

use std::collections::{HashMap, VecDeque};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use self::buffer::TextBuffer;
use super::{CanvasService, Rect, TerminalKeyCode, TerminalKeyEvent, TextStyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InputId(pub u64);

#[derive(Clone, Debug)]
pub struct InputOptions {
  pub initial_text: String,
  pub max_chars: Option<usize>,
  pub multiline: bool,
}

#[derive(Clone, Debug)]
pub struct InputDrawParams {
  pub rect: Rect,
  pub placeholder: String,
  pub text_style: TextStyle,
  pub placeholder_style: TextStyle,
  pub cursor_style: TextStyle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputEvent {
  Focused { id: InputId },
  Blurred { id: InputId },
  Changed { id: InputId, value: String },
  Submit { id: InputId, value: String },
  Cancel { id: InputId, value: String },
}

struct TextInputState {
  buffer: TextBuffer,
  _multiline: bool,
}

pub struct UiObjectPool {
  next_input_id: u64,
  inputs: HashMap<InputId, TextInputState>,
  input_events: VecDeque<InputEvent>,
}

impl UiObjectPool {
  pub fn new() -> Self {
    Self {
      next_input_id: 1,
      inputs: HashMap::new(),
      input_events: VecDeque::new(),
    }
  }

  pub fn create_input(&mut self, options: InputOptions) -> InputId {
    let id = InputId(self.next_input_id);
    self.next_input_id += 1;
    self.inputs.insert(
      id,
      TextInputState {
        buffer: TextBuffer::new(options.initial_text, options.max_chars),
        _multiline: options.multiline,
      },
    );
    id
  }

  /// active 输入对象必须先通过 TextInputService blur，再删除。
  pub fn remove_input(&mut self, id: InputId) -> bool {
    let removed = self.inputs.remove(&id).is_some();
    if removed {
      self.input_events.retain(|event| event_id(event) != id);
    }
    removed
  }

  pub fn get_input_text(&self, id: InputId) -> Option<&str> {
    self.inputs.get(&id).map(|state| state.buffer.text())
  }

  pub fn set_input_text(&mut self, id: InputId, text: String) -> bool {
    let Some(state) = self.inputs.get_mut(&id) else {
      return false;
    };
    if !state.buffer.set_text(text) {
      return false;
    }
    let value = state.buffer.text().to_string();
    self
      .input_events
      .push_back(InputEvent::Changed { id, value });
    true
  }

  pub fn clear_input(&mut self, id: InputId) -> bool {
    self.set_input_text(id, String::new())
  }

  pub fn input_exists(&self, id: InputId) -> bool {
    self.inputs.contains_key(&id)
  }

  pub fn take_input_events(&mut self, id: InputId) -> Vec<InputEvent> {
    let mut selected = Vec::new();
    let mut others = VecDeque::new();
    while let Some(event) = self.input_events.pop_front() {
      if event_id(&event) == id {
        selected.push(event);
      } else {
        others.push_back(event);
      }
    }
    self.input_events = others;
    selected
  }

  pub fn draw_input(
    &self,
    id: InputId,
    params: &InputDrawParams,
    canvas: &mut CanvasService,
    active_input: Option<InputId>,
  ) -> Option<(u16, u16)> {
    if params.rect.width == 0 || params.rect.height == 0 {
      return None;
    }
    let Some(state) = self.inputs.get(&id) else {
      return None;
    };

    let active = active_input == Some(id);
    if state.buffer.text().is_empty() {
      if active {
        canvas.styled_text(
          params.rect.x,
          params.rect.y,
          "█",
          params.cursor_style.clone(),
        );
        return (params.rect.x < canvas.width() && params.rect.y < canvas.height())
          .then_some((params.rect.x, params.rect.y));
      } else {
        draw_prefix(
          canvas,
          params.rect.x,
          params.rect.y,
          &params.placeholder,
          params.rect.width,
          params.placeholder_style.clone(),
        );
      }
      return None;
    }

    if !active {
      draw_prefix(
        canvas,
        params.rect.x,
        params.rect.y,
        state.buffer.text(),
        params.rect.width,
        params.text_style.clone(),
      );
      return None;
    }

    let (before, after) = visible_around_cursor(
      state.buffer.text(),
      state.buffer.cursor(),
      params.rect.width.saturating_sub(1) as usize,
    );
    let before_width = UnicodeWidthStr::width(before.as_str()) as u16;
    canvas.styled_text(
      params.rect.x,
      params.rect.y,
      &before,
      params.text_style.clone(),
    );
    let cursor_x = params.rect.x.saturating_add(before_width);
    canvas.styled_text(cursor_x, params.rect.y, "█", params.cursor_style.clone());
    canvas.styled_text(
      cursor_x.saturating_add(1),
      params.rect.y,
      &after,
      params.text_style.clone(),
    );
    (cursor_x < canvas.width() && params.rect.y < canvas.height())
      .then_some((cursor_x, params.rect.y))
  }
}

pub struct TextInputService {
  active_input: Option<InputId>,
}

impl TextInputService {
  pub fn new() -> Self {
    Self { active_input: None }
  }

  pub fn is_active(&self) -> bool {
    self.active_input.is_some()
  }

  pub fn active_input(&self) -> Option<InputId> {
    self.active_input
  }

  pub fn focus_input(&mut self, pool: &mut UiObjectPool, id: InputId) -> bool {
    if self.active_input.is_some() || !pool.input_exists(id) {
      return false;
    }
    self.active_input = Some(id);
    pool.input_events.push_back(InputEvent::Focused { id });
    true
  }

  pub fn blur_input(&mut self, pool: &mut UiObjectPool) -> bool {
    let Some(id) = self.active_input else {
      return false;
    };
    if !pool.input_exists(id) {
      return false;
    }
    self.active_input = None;
    pool.input_events.push_back(InputEvent::Blurred { id });
    true
  }

  pub fn route_terminal_key(&mut self, pool: &mut UiObjectPool, key: TerminalKeyEvent) {
    let Some(id) = self.active_input else {
      return;
    };
    let Some(state) = pool.inputs.get_mut(&id) else {
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
      TerminalKeyCode::Enter => {
        pool.input_events.push_back(InputEvent::Submit {
          id,
          value: state.buffer.text().to_string(),
        });
        return;
      }
      TerminalKeyCode::Esc => {
        pool.input_events.push_back(InputEvent::Cancel {
          id,
          value: state.buffer.text().to_string(),
        });
        return;
      }
    };

    if changed
      && matches!(
        key.code,
        TerminalKeyCode::Char(_) | TerminalKeyCode::Backspace | TerminalKeyCode::Delete
      )
    {
      pool.input_events.push_back(InputEvent::Changed {
        id,
        value: state.buffer.text().to_string(),
      });
    }
  }
}

fn event_id(event: &InputEvent) -> InputId {
  match event {
    InputEvent::Focused { id }
    | InputEvent::Blurred { id }
    | InputEvent::Changed { id, .. }
    | InputEvent::Submit { id, .. }
    | InputEvent::Cancel { id, .. } => *id,
  }
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

fn visible_around_cursor(text: &str, cursor: usize, width: usize) -> (String, String) {
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
  let mut used = 0;
  while start > 0 {
    let grapheme_width = UnicodeWidthStr::width(graphemes[start - 1]);
    if used + grapheme_width > width {
      break;
    }
    start -= 1;
    used += grapheme_width;
  }

  let mut end = cursor_index;
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
    graphemes[cursor_index..end].concat(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  fn options(text: &str) -> InputOptions {
    InputOptions {
      initial_text: text.to_string(),
      max_chars: None,
      multiline: false,
    }
  }

  fn draw_params(width: u16, placeholder: &str) -> InputDrawParams {
    InputDrawParams {
      rect: Rect {
        x: 0,
        y: 0,
        width,
        height: 1,
      },
      placeholder: placeholder.to_string(),
      text_style: TextStyle::default(),
      placeholder_style: TextStyle::default(),
      cursor_style: TextStyle::default(),
    }
  }

  #[test]
  fn pool_ids_are_unique_and_events_are_scoped() {
    let mut pool = UiObjectPool::new();
    let first = pool.create_input(options(""));
    let second = pool.create_input(options(""));
    assert_eq!((first, second), (InputId(1), InputId(2)));

    assert!(pool.set_input_text(first, "a".to_string()));
    assert!(!pool.set_input_text(first, "a".to_string()));
    assert!(pool.set_input_text(second, "b".to_string()));
    assert!(pool.remove_input(first));
    assert!(!pool.input_exists(first));
    assert!(pool.take_input_events(first).is_empty());
    assert_eq!(pool.take_input_events(second).len(), 1);
    assert!(pool.clear_input(second));
    assert_eq!(pool.get_input_text(second), Some(""));
    assert!(matches!(
      pool.take_input_events(second).as_slice(),
      [InputEvent::Changed { value, .. }] if value.is_empty()
    ));
  }

  #[test]
  fn focus_requires_valid_inactive_input() {
    let mut pool = UiObjectPool::new();
    let id = pool.create_input(options(""));
    let mut service = TextInputService::new();

    assert!(!service.focus_input(&mut pool, InputId(99)));
    assert!(service.focus_input(&mut pool, id));
    assert!(!service.focus_input(&mut pool, id));
    assert!(service.blur_input(&mut pool));
    assert!(!service.blur_input(&mut pool));
    assert_eq!(
      pool.take_input_events(id),
      vec![InputEvent::Focused { id }, InputEvent::Blurred { id }]
    );
  }

  #[test]
  fn service_edits_and_emits_only_real_changes() {
    let mut pool = UiObjectPool::new();
    let id = pool.create_input(options(""));
    let mut service = TextInputService::new();
    service.focus_input(&mut pool, id);
    pool.take_input_events(id);

    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Char('我'),
      },
    );
    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Left,
      },
    );
    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Backspace,
      },
    );
    assert_eq!(pool.get_input_text(id), Some("我"));
    assert_eq!(pool.take_input_events(id).len(), 1);

    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Enter,
      },
    );
    service.route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Esc,
      },
    );
    let events = pool.take_input_events(id);
    assert!(matches!(events[0], InputEvent::Submit { .. }));
    assert!(matches!(events[1], InputEvent::Cancel { .. }));
    assert!(service.is_active());
  }

  #[test]
  fn inactive_service_ignores_terminal_key() {
    let mut pool = UiObjectPool::new();
    let id = pool.create_input(options(""));
    TextInputService::new().route_terminal_key(
      &mut pool,
      TerminalKeyEvent {
        code: TerminalKeyCode::Char('a'),
      },
    );
    assert_eq!(pool.get_input_text(id), Some(""));
  }

  #[test]
  fn draw_input_handles_placeholder_cursor_scroll_and_raw_prefix() {
    let mut pool = UiObjectPool::new();
    let empty = pool.create_input(options(""));
    let raw = pool.create_input(options("f%abc"));
    let cjk = pool.create_input(options("你好abc"));
    let mut canvas = CanvasService::new();

    assert_eq!(
      pool.draw_input(empty, &draw_params(5, "hint"), &mut canvas, None),
      None
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "h");
    canvas.clear();
    assert_eq!(
      pool.draw_input(empty, &draw_params(5, "hint"), &mut canvas, Some(empty),),
      Some((0, 0))
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "█");

    canvas.clear();
    assert_eq!(
      pool.draw_input(raw, &draw_params(6, ""), &mut canvas, None),
      None
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "f");
    assert_eq!(canvas.cell_at(1, 0).unwrap().text, "%");

    canvas.clear();
    assert_eq!(
      pool.draw_input(cjk, &draw_params(4, ""), &mut canvas, Some(cjk)),
      Some((3, 0))
    );
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "a");
    assert_eq!(canvas.cell_at(3, 0).unwrap().text, "█");
  }
}
