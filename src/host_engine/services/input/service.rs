use std::time::Duration;

use crossterm::event::{
  self,
  Event,
  MouseButton as CrosstermMouseButton,
  MouseEvent,
  MouseEventKind,
  poll,
};

use super::{
  InputEvent,
  InputEventQueue,
  KeyboardFrameState,
  KeyboardInputEvent,
  KeyboardInputKind,
  MouseButton,
  MouseInputEvent,
  MouseInputKind,
  RawInputEvent,
  RawInputSource,
  WindowInputEvent,
};

// 辅助：crossterm KeyEventKind → KeyboardInputKind
fn keyboard_kind_from_crossterm(kind: crossterm::event::KeyEventKind) -> KeyboardInputKind {
  match kind {
    crossterm::event::KeyEventKind::Press => KeyboardInputKind::Press,
    crossterm::event::KeyEventKind::Release => KeyboardInputKind::Release,
    crossterm::event::KeyEventKind::Repeat => KeyboardInputKind::Repeat,
  }
}

// 辅助：crossterm MouseButton → MouseButton
fn mouse_button_from_crossterm(button: CrosstermMouseButton) -> Option<MouseButton> {
  match button {
    CrosstermMouseButton::Left => Some(MouseButton::Left),
    CrosstermMouseButton::Right => Some(MouseButton::Right),
    CrosstermMouseButton::Middle => Some(MouseButton::Middle),
  }
}

// 辅助：crossterm MouseEventKind → MouseInputKind
fn mouse_kind_from_crossterm(kind: MouseEventKind) -> Option<MouseInputKind> {
  match kind {
    MouseEventKind::Moved => Some(MouseInputKind::Move),
    MouseEventKind::Down(button) => mouse_button_from_crossterm(button).map(MouseInputKind::Down),
    MouseEventKind::Up(button) => mouse_button_from_crossterm(button).map(MouseInputKind::Up),
    MouseEventKind::Drag(button) => mouse_button_from_crossterm(button).map(MouseInputKind::Drag),
    MouseEventKind::ScrollUp => Some(MouseInputKind::ScrollUp),
    MouseEventKind::ScrollDown => Some(MouseInputKind::ScrollDown),
    MouseEventKind::ScrollLeft => Some(MouseInputKind::ScrollLeft),
    MouseEventKind::ScrollRight => Some(MouseInputKind::ScrollRight),
  }
}

// 辅助：crossterm MouseEvent → MouseInputEvent
fn mouse_event_from_crossterm(event: MouseEvent) -> Option<MouseInputEvent> {
  mouse_kind_from_crossterm(event.kind)
    .map(|kind| MouseInputEvent::new(event.column, event.row, kind))
}

pub struct InputService {
  queue: InputEventQueue,
  keyboard_state: KeyboardFrameState,
}

impl InputService {
  pub fn new() -> Self {
    Self {
      queue: InputEventQueue::new(),
      keyboard_state: KeyboardFrameState::new(),
    }
  }

  pub fn keyboard_state(&self) -> &KeyboardFrameState {
    &self.keyboard_state
  }

  pub fn keyboard_state_mut(&mut self) -> &mut KeyboardFrameState {
    &mut self.keyboard_state
  }

  pub fn queued_event_count(&self) -> usize {
    self.queue.len()
  }

  pub fn clear_events(&mut self) {
    self.queue.clear();
  }

  pub fn push_raw_event(&mut self, event: RawInputEvent) {
    match event {
      RawInputEvent::Keyboard { event, .. } => {
        self.keyboard_state.apply_event(event);
        self.queue.push(InputEvent::Keyboard(event));
      }
      RawInputEvent::Mouse { .. } | RawInputEvent::Window { .. } => {
        self.queue.push(event.into_input_event());
      }
    }
  }

  pub fn push_raw_events<I>(&mut self, events: I)
  where
    I: IntoIterator<Item = RawInputEvent>,
  {
    for event in events {
      self.push_raw_event(event);
    }
  }

  pub fn begin_frame(&mut self) {
    self.keyboard_state.begin_frame();
  }

  pub fn poll_terminal_events(&mut self) {
    while poll(Duration::ZERO).unwrap_or(false) {
      match event::read() {
        Ok(Event::Key(key_event)) => {
          let keyboard_event = KeyboardInputEvent::new(
            key_event.code,
            key_event.modifiers,
            keyboard_kind_from_crossterm(key_event.kind),
          );
          self.push_raw_event(RawInputEvent::Keyboard {
            source: RawInputSource::Terminal,
            event: keyboard_event,
          });
        }
        Ok(Event::Mouse(mouse_event)) => {
          if let Some(mouse_event) = mouse_event_from_crossterm(mouse_event) {
            self.push_raw_event(RawInputEvent::Mouse {
              source: RawInputSource::Terminal,
              event: mouse_event,
            });
          }
        }
        Ok(Event::Resize(width, height)) => {
          self.push_raw_event(RawInputEvent::Window {
            source: RawInputSource::Terminal,
            event: WindowInputEvent::Resize { width, height },
          });
        }
        Ok(Event::FocusGained) => {
          self.push_raw_event(RawInputEvent::Window {
            source: RawInputSource::Terminal,
            event: WindowInputEvent::FocusGained,
          });
        }
        Ok(Event::FocusLost) => {
          self.push_raw_event(RawInputEvent::Window {
            source: RawInputSource::Terminal,
            event: WindowInputEvent::FocusLost,
          });
        }
        _ => {}
      }
    }
  }

  pub fn poll(&mut self) {
    self.begin_frame();
    self.poll_terminal_events();
  }

  // 消费下一个事件
  pub fn next_event(&mut self) -> Option<InputEvent> {
    self.queue.pop()
  }
}
