use std::collections::VecDeque;
use std::time::Duration;

use crossterm::event::{
  self,
  Event,
  KeyCode,
  MouseButton as CrosstermMouseButton,
  MouseEvent,
  MouseEventKind,
  poll,
};

use super::{
  InputEvent,
  KeyboardInputEvent,
  KeyboardInputKind,
  MouseButton,
  MouseInputEvent,
  MouseInputKind,
  WindowInputEvent,
};

// 按键输入（兼容性类型，后续由 InputEvent::Keyboard 直接替代）
#[derive(Clone, Debug)]
pub struct KeyInput {
  pub code: KeyCode,
  pub kind: KeyEventKind,
}

// 按键输入状态
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEventKind {
  Press,
  Release,
  Repeat,
}

// 辅助：crossterm KeyEventKind → KeyboardInputKind
fn keyboard_kind_from_crossterm(kind: crossterm::event::KeyEventKind) -> KeyboardInputKind {
  match kind {
    crossterm::event::KeyEventKind::Press => KeyboardInputKind::Press,
    crossterm::event::KeyEventKind::Release => KeyboardInputKind::Release,
    crossterm::event::KeyEventKind::Repeat => KeyboardInputKind::Repeat,
  }
}

// 辅助：KeyboardInputKind → 遗留 KeyEventKind
fn legacy_key_kind_from_keyboard(kind: KeyboardInputKind) -> KeyEventKind {
  match kind {
    KeyboardInputKind::Press => KeyEventKind::Press,
    KeyboardInputKind::Release => KeyEventKind::Release,
    KeyboardInputKind::Repeat => KeyEventKind::Repeat,
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

impl From<KeyboardInputEvent> for KeyInput {
  fn from(event: KeyboardInputEvent) -> Self {
    Self {
      code: event.code,
      kind: legacy_key_kind_from_keyboard(event.kind),
    }
  }
}

pub struct InputService {
  queue: VecDeque<InputEvent>,
}

impl InputService {
  pub fn new() -> Self {
    Self {
      queue: VecDeque::new(),
    }
  }

  // 收集所有待处理事件（键盘、鼠标、窗口）
  pub fn poll(&mut self) {
    while poll(Duration::ZERO).unwrap_or(false) {
      match event::read() {
        Ok(Event::Key(key_event)) => {
          self
            .queue
            .push_back(InputEvent::Keyboard(KeyboardInputEvent::new(
              key_event.code,
              key_event.modifiers,
              keyboard_kind_from_crossterm(key_event.kind),
            )));
        }
        Ok(Event::Mouse(mouse_event)) => {
          if let Some(mouse_event) = mouse_event_from_crossterm(mouse_event) {
            self.queue.push_back(InputEvent::Mouse(mouse_event));
          }
        }
        Ok(Event::Resize(width, height)) => {
          self
            .queue
            .push_back(InputEvent::Window(WindowInputEvent::Resize { width, height }));
        }
        Ok(Event::FocusGained) => {
          self
            .queue
            .push_back(InputEvent::Window(WindowInputEvent::FocusGained));
        }
        Ok(Event::FocusLost) => {
          self
            .queue
            .push_back(InputEvent::Window(WindowInputEvent::FocusLost));
        }
        _ => {}
      }
    }
  }

  // 获取下一个按键（头部出队并返回兼容性类型）
  pub fn next_key(&mut self) -> Option<KeyInput> {
    match self.queue.pop_front() {
      Some(InputEvent::Keyboard(key)) => Some(KeyInput::from(key)),
      _ => None,
    }
  }

  // 下个事件
  pub fn next_event(&mut self) -> Option<InputEvent> {
    self.queue.pop_front()
  }

  // 消费按键事件
  pub fn consume_key(&mut self, code: KeyCode) -> bool {
    let matched = self.queue.front().is_some_and(|event| {
      match event {
        InputEvent::Keyboard(key) => {
          key.code == code
            && matches!(key.kind, KeyboardInputKind::Press | KeyboardInputKind::Repeat)
        }
        _ => false,
      }
    });

    if matched {
      self.queue.pop_front();
      return true;
    }

    false
  }

  // 消费尺寸变化事件
  pub fn consume_resize(&mut self) -> Option<(u16, u16)> {
    let matched = self.queue.front().and_then(|event| {
      match event {
        InputEvent::Window(WindowInputEvent::Resize { width, height }) => Some((*width, *height)),
        _ => None,
      }
    });

    if matched.is_some() {
      self.queue.pop_front();
    }

    matched
  }
}
