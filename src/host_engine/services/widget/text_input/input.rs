use std::time::{Duration, Instant};

use super::layout::{VisualLayout, cursor_from_point, move_line_edge, move_vertical};
use super::service::TextInputService;
use super::state::{ActiveTextInput, DragSelection, TextInputActive};
use super::types::{TextInputEvent, TextInputMode};
use crate::host_engine::services::ui::{UiComponentEvent, UiObjectPool};
use crate::host_engine::services::{
  ClipboardService, MouseButton, MouseEvent, MouseEventKind, TerminalKeyCode, TerminalKeyEvent,
};

const DRAG_SCROLL_INTERVAL: Duration = Duration::from_millis(100);

impl TextInputService {
  /// 将终端按键事件路由到当前聚焦的输入组件，执行编辑操作。
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

  /// 将鼠标事件路由到对应输入组件，处理点击聚焦和拖拽选区。
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
}
