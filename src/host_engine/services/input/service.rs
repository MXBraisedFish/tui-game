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
  ExternalRawInputQueue,
  ExternalRawInputSender,
  GlobalKeyboardControl,
  GlobalKeyboardListener,
  InputEvent,
  InputEventQueue,
  KeyboardFrameState,
  KeyboardInputBackend,
  KeyboardInputEvent,
  KeyboardInputKind,
  MouseButton,
  MouseInputEvent,
  MouseInputKind,
  RawInputEvent,
  RawInputSource,
  WindowInputEvent,
  physical_key_from_crossterm,
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
  external_raw_queue: ExternalRawInputQueue,
  global_keyboard: GlobalKeyboardListener,
  keyboard_backend: KeyboardInputBackend,
  keyboard_state: KeyboardFrameState,
}

impl InputService {
  pub fn new() -> Self {
    Self {
      queue: InputEventQueue::new(),
      external_raw_queue: ExternalRawInputQueue::new(),
      global_keyboard: GlobalKeyboardListener::new(),
      keyboard_backend: KeyboardInputBackend::Terminal,
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

  fn should_accept_raw_input_event(&self, event: &RawInputEvent) -> bool {
    match event {
      RawInputEvent::Keyboard { source, .. } => {
        match source {
          RawInputSource::Terminal => self.uses_terminal_keyboard_backend(),
          RawInputSource::GlobalKeyboard => self.uses_global_keyboard_backend(),
        }
      }
      RawInputEvent::Mouse { source, .. } | RawInputEvent::Window { source, .. } => {
        matches!(source, RawInputSource::Terminal)
      }
    }
  }

  pub fn push_raw_event(&mut self, event: RawInputEvent) {
    if !self.should_accept_raw_input_event(&event) {
      return;
    }

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

  pub fn push_external_raw_event(&mut self, event: RawInputEvent) {
    self.external_raw_queue.push(event);
  }

  pub fn push_external_raw_events<I>(&mut self, events: I)
  where
    I: IntoIterator<Item = RawInputEvent>,
  {
    for event in events {
      self.push_external_raw_event(event);
    }
  }

  pub fn queued_external_raw_event_count(&self) -> usize {
    self.external_raw_queue.len()
  }

  pub fn clear_external_raw_events(&mut self) {
    self.external_raw_queue.clear();
  }

  pub fn external_raw_input_sender(&self) -> ExternalRawInputSender {
    self.external_raw_queue.sender()
  }

  pub fn start_global_keyboard_listener(&mut self) {
    let sender = self.external_raw_input_sender();
    self.global_keyboard.start(sender);
  }

  pub fn enable_global_keyboard(&self) {
    self.global_keyboard.enable();
  }

  pub fn disable_global_keyboard(&self) {
    self.global_keyboard.disable();
  }

  pub fn global_keyboard_control(&self) -> GlobalKeyboardControl {
    self.global_keyboard.control()
  }

  pub fn is_global_keyboard_started(&self) -> bool {
    self.global_keyboard.is_started()
  }

  pub fn keyboard_backend(&self) -> KeyboardInputBackend {
    self.keyboard_backend
  }

  pub fn use_terminal_keyboard_backend(&mut self) {
    self.keyboard_backend = KeyboardInputBackend::Terminal;
    self.disable_global_keyboard();
    self.clear_external_raw_events();
    self.keyboard_state.clear();
  }

  pub fn use_global_keyboard_backend(&mut self) {
    self.keyboard_backend = KeyboardInputBackend::Global;
    self.clear_external_raw_events();
    self.keyboard_state.clear();
    self.start_global_keyboard_listener();
    self.enable_global_keyboard();
  }

  pub fn uses_terminal_keyboard_backend(&self) -> bool {
    matches!(self.keyboard_backend, KeyboardInputBackend::Terminal)
  }

  pub fn uses_global_keyboard_backend(&self) -> bool {
    matches!(self.keyboard_backend, KeyboardInputBackend::Global)
  }

  pub fn suspend_keyboard_input(&mut self) {
    self.disable_global_keyboard();
    self.clear_external_raw_events();
    self.keyboard_state.clear();
  }

  pub fn resume_keyboard_input(&mut self) {
    if self.uses_global_keyboard_backend() {
      self.enable_global_keyboard();
    }
  }

  pub fn begin_frame(&mut self) {
    self.keyboard_state.begin_frame();
  }

  pub fn poll_terminal_events(&mut self) {
    while poll(Duration::ZERO).unwrap_or(false) {
      match event::read() {
        Ok(Event::Key(key_event)) => {
          if self.uses_terminal_keyboard_backend() {
            if let Some(key) = physical_key_from_crossterm(key_event.code) {
              let keyboard_event = KeyboardInputEvent::new(
                key,
                key_event.modifiers,
                keyboard_kind_from_crossterm(key_event.kind),
              );
              self.push_raw_event(RawInputEvent::Keyboard {
                source: RawInputSource::Terminal,
                event: keyboard_event,
              });
            }
          }
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

  pub fn drain_external_raw_events(&mut self) {
    while let Some(event) = self.external_raw_queue.pop() {
      self.push_raw_event(event);
    }
  }

  pub fn poll(&mut self) {
    self.begin_frame();
    self.poll_terminal_events();
    self.drain_external_raw_events();
  }

  // 消费下一个事件
  pub fn next_event(&mut self) -> Option<InputEvent> {
    self.queue.pop()
  }
}
