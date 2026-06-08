use std::collections::HashSet;
use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};
use std::thread;

use crossbeam_channel::{Receiver, Sender, unbounded};

use rdev::{Event, EventType, Key as RdevKey, listen};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Key {
  Esc,

  Enter,
  Tab,
  Backspace,
  Space,

  Up,
  Down,
  Left,
  Right,

  Home,
  End,
  PageUp,
  PageDown,
  Insert,
  Delete,

  Fn(u8),

  Num(u8),
  Numpad(u8),

  A,
  B,
  C,
  D,
  E,
  F,
  G,
  H,
  I,
  J,
  K,
  L,
  M,
  N,
  O,
  P,
  Q,
  R,
  S,
  T,
  U,
  V,
  W,
  X,
  Y,
  Z,

  LeftCtrl,
  RightCtrl,
  LeftShift,
  RightShift,
  LeftAlt,
  RightAlt,
  LeftMeta,
  RightMeta,

  CapsLock,
  NumLock,
  ScrollLock,

  PrintScreen,
  Pause,

  BackQuote,
  Minus,
  Equal,
  LeftBracket,
  RightBracket,
  BackSlash,
  Semicolon,
  Quote,
  Comma,
  Dot,
  Slash,

  NumpadAdd,
  NumpadSubtract,
  NumpadMultiply,
  NumpadDivide,
  NumpadEnter,
  NumpadDelete,

  Unknown(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEventKind {
  Press,
  Release,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
  pub key: Key,
  pub kind: KeyEventKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyState {
  Pressed,
  Held,
  Released,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyPattern {
  Single(Key),
  Combo(Key, Key),
}

impl KeyPattern {
  pub fn normalized(self) -> Self {
    match self {
      KeyPattern::Single(key) => KeyPattern::Single(key),

      KeyPattern::Combo(first, second) => {
        if first <= second {
          KeyPattern::Combo(first, second)
        } else {
          KeyPattern::Combo(second, first)
        }
      }
    }
  }

  pub fn has_consumed_key(&self, consumed_keys: &HashSet<Key>) -> bool {
    match self.normalized() {
      KeyPattern::Single(key) => consumed_keys.contains(&key),

      KeyPattern::Combo(first, second) => {
        consumed_keys.contains(&first) || consumed_keys.contains(&second)
      }
    }
  }

  pub fn consume_keys(&self, consumed_keys: &mut HashSet<Key>) {
    match self.normalized() {
      KeyPattern::Single(key) => {
        consumed_keys.insert(key);
      }

      KeyPattern::Combo(first, second) => {
        consumed_keys.insert(first);
        consumed_keys.insert(second);
      }
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyBinding {
  pub pattern: KeyPattern,
  pub action: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEventType {
  Keyboard,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputActionEvent {
  pub event_type: InputEventType,
  pub action: String,
  pub state: KeyState,
}

pub struct InputService {
  sender: Sender<KeyEvent>,
  receiver: Receiver<KeyEvent>,
  action_sender: Sender<InputActionEvent>,
  action_receiver: Receiver<InputActionEvent>,
  listener_started: Arc<AtomicBool>,
  held_keys: HashSet<Key>,
  pressed_keys: HashSet<Key>,
  released_keys: HashSet<Key>,
  bindings: Vec<KeyBinding>,
}

impl InputService {
  pub fn new() -> Self {
    let (sender, receiver) = unbounded();
    let (action_sender, action_receiver) = unbounded();

    Self {
      sender,
      receiver,
      action_sender,
      action_receiver,
      listener_started: Arc::new(AtomicBool::new(false)),
      held_keys: HashSet::new(),
      pressed_keys: HashSet::new(),
      released_keys: HashSet::new(),
      bindings: Vec::new(),
    }
  }

  pub fn start_key_listener(&self) {
    if self.listener_started.swap(true, Ordering::SeqCst) {
      return;
    }

    let sender = self.sender.clone();

    thread::spawn(move || {
      let callback = move |event: Event| {
        if let Some(key_event) = key_event_from_rdev(event) {
          let _ = sender.send(key_event);
        }
      };
      let _ = listen(callback);
    });
  }

  pub fn begin_frame(&mut self) {
    self.pressed_keys.clear();
    self.released_keys.clear();
  }

  pub fn poll(&mut self) {
    while let Ok(event) = self.receiver.try_recv() {
      self.apply_key_event(event);
    }
  }

  pub fn is_down(&self, key: Key) -> bool {
    self.held_keys.contains(&key)
  }

  pub fn was_pressed(&self, key: Key) -> bool {
    self.pressed_keys.contains(&key)
  }

  pub fn was_released(&self, key: Key) -> bool {
    self.released_keys.contains(&key)
  }

  pub fn key_state(&self, key: Key) -> Option<KeyState> {
    if self.pressed_keys.contains(&key) {
      return Some(KeyState::Pressed);
    }

    if self.released_keys.contains(&key) {
      return Some(KeyState::Released);
    }

    if self.held_keys.contains(&key) {
      return Some(KeyState::Held);
    }

    None
  }

  pub fn clear(&mut self) {
    self.held_keys.clear();
    self.pressed_keys.clear();
    self.released_keys.clear();
  }

  pub fn load_key_bindings(&mut self, bindings: Vec<KeyBinding>) {
    self.bindings = bindings
      .into_iter()
      .map(|binding| KeyBinding {
        pattern: binding.pattern.normalized(),
        action: binding.action,
      })
      .collect();
  }

  pub fn key_bindings(&self) -> &[KeyBinding] {
    &self.bindings
  }

  fn pattern_state(&self, pattern: KeyPattern) -> Option<KeyState> {
    match pattern.normalized() {
      KeyPattern::Single(key) => self.key_state(key),

      KeyPattern::Combo(first, second) => self.combo_state(first, second),
    }
  }

  fn combo_state(&self, first: Key, second: Key) -> Option<KeyState> {
    let first_released = self.was_released(first);
    let second_released = self.was_released(second);

    let first_pressed = self.was_pressed(first);
    let second_pressed = self.was_pressed(second);

    let first_down = self.is_down(first);
    let second_down = self.is_down(second);

    if first_released && self.key_is_active_or_changed(second) {
      return Some(KeyState::Released);
    }

    if second_released && self.key_is_active_or_changed(first) {
      return Some(KeyState::Released);
    }

    if first_pressed && second_down {
      return Some(KeyState::Pressed);
    }

    if second_pressed && first_down {
      return Some(KeyState::Pressed);
    }

    if first_down && second_down {
      return Some(KeyState::Held);
    }

    None
  }

  fn key_is_active_or_changed(&self, key: Key) -> bool {
    self.is_down(key) || self.was_pressed(key) || self.was_released(key)
  }

  pub fn collect_action_events(&self) -> Vec<InputActionEvent> {
    let mut events = Vec::new();
    let mut consumed_keys = HashSet::new();

    for binding in &self.bindings {
      let pattern = binding.pattern.normalized();

      if pattern.has_consumed_key(&consumed_keys) {
        continue;
      }

      if let Some(state) = self.pattern_state(pattern) {
        events.push(InputActionEvent {
          event_type: InputEventType::Keyboard,
          action: binding.action.clone(),
          state,
        });

        pattern.consume_keys(&mut consumed_keys);
      }
    }

    events
  }

  pub fn dispatch_action_events(&self) {
    for event in self.collect_action_events() {
      let _ = self.action_sender.send(event);
    }
  }

  pub fn next_action_event(&self) -> Option<InputActionEvent> {
    self.action_receiver.try_recv().ok()
  }

  fn apply_key_event(&mut self, event: KeyEvent) {
    match event.kind {
      KeyEventKind::Press => {
        if self.held_keys.insert(event.key) {
          self.pressed_keys.insert(event.key);
        }
      }

      KeyEventKind::Release => {
        if self.held_keys.remove(&event.key) {
          self.released_keys.insert(event.key);
        }
      }
    }
  }
}

fn key_from_rdev(key: RdevKey) -> Option<Key> {
  match key {
    RdevKey::Escape => Some(Key::Esc),

    RdevKey::Return => Some(Key::Enter),
    RdevKey::Tab => Some(Key::Tab),
    RdevKey::Backspace => Some(Key::Backspace),
    RdevKey::Space => Some(Key::Space),

    RdevKey::UpArrow => Some(Key::Up),
    RdevKey::DownArrow => Some(Key::Down),
    RdevKey::LeftArrow => Some(Key::Left),
    RdevKey::RightArrow => Some(Key::Right),

    RdevKey::Home => Some(Key::Home),
    RdevKey::End => Some(Key::End),
    RdevKey::PageUp => Some(Key::PageUp),
    RdevKey::PageDown => Some(Key::PageDown),
    RdevKey::Insert => Some(Key::Insert),
    RdevKey::Delete => Some(Key::Delete),

    RdevKey::F1 => Some(Key::Fn(1)),
    RdevKey::F2 => Some(Key::Fn(2)),
    RdevKey::F3 => Some(Key::Fn(3)),
    RdevKey::F4 => Some(Key::Fn(4)),
    RdevKey::F5 => Some(Key::Fn(5)),
    RdevKey::F6 => Some(Key::Fn(6)),
    RdevKey::F7 => Some(Key::Fn(7)),
    RdevKey::F8 => Some(Key::Fn(8)),
    RdevKey::F9 => Some(Key::Fn(9)),
    RdevKey::F10 => Some(Key::Fn(10)),
    RdevKey::F11 => Some(Key::Fn(11)),
    RdevKey::F12 => Some(Key::Fn(12)),

    RdevKey::Num0 => Some(Key::Num(0)),
    RdevKey::Num1 => Some(Key::Num(1)),
    RdevKey::Num2 => Some(Key::Num(2)),
    RdevKey::Num3 => Some(Key::Num(3)),
    RdevKey::Num4 => Some(Key::Num(4)),
    RdevKey::Num5 => Some(Key::Num(5)),
    RdevKey::Num6 => Some(Key::Num(6)),
    RdevKey::Num7 => Some(Key::Num(7)),
    RdevKey::Num8 => Some(Key::Num(8)),
    RdevKey::Num9 => Some(Key::Num(9)),

    RdevKey::KeyA => Some(Key::A),
    RdevKey::KeyB => Some(Key::B),
    RdevKey::KeyC => Some(Key::C),
    RdevKey::KeyD => Some(Key::D),
    RdevKey::KeyE => Some(Key::E),
    RdevKey::KeyF => Some(Key::F),
    RdevKey::KeyG => Some(Key::G),
    RdevKey::KeyH => Some(Key::H),
    RdevKey::KeyI => Some(Key::I),
    RdevKey::KeyJ => Some(Key::J),
    RdevKey::KeyK => Some(Key::K),
    RdevKey::KeyL => Some(Key::L),
    RdevKey::KeyM => Some(Key::M),
    RdevKey::KeyN => Some(Key::N),
    RdevKey::KeyO => Some(Key::O),
    RdevKey::KeyP => Some(Key::P),
    RdevKey::KeyQ => Some(Key::Q),
    RdevKey::KeyR => Some(Key::R),
    RdevKey::KeyS => Some(Key::S),
    RdevKey::KeyT => Some(Key::T),
    RdevKey::KeyU => Some(Key::U),
    RdevKey::KeyV => Some(Key::V),
    RdevKey::KeyW => Some(Key::W),
    RdevKey::KeyX => Some(Key::X),
    RdevKey::KeyY => Some(Key::Y),
    RdevKey::KeyZ => Some(Key::Z),

    RdevKey::ControlLeft => Some(Key::LeftCtrl),
    RdevKey::ControlRight => Some(Key::RightCtrl),

    RdevKey::ShiftLeft => Some(Key::LeftShift),
    RdevKey::ShiftRight => Some(Key::RightShift),

    RdevKey::Alt => Some(Key::LeftAlt),
    RdevKey::AltGr => Some(Key::RightAlt),

    RdevKey::MetaLeft => Some(Key::LeftMeta),
    RdevKey::MetaRight => Some(Key::RightMeta),

    RdevKey::CapsLock => Some(Key::CapsLock),
    RdevKey::NumLock => Some(Key::NumLock),
    RdevKey::ScrollLock => Some(Key::ScrollLock),

    RdevKey::PrintScreen => Some(Key::PrintScreen),
    RdevKey::Pause => Some(Key::Pause),

    RdevKey::BackQuote => Some(Key::BackQuote),
    RdevKey::Minus => Some(Key::Minus),
    RdevKey::Equal => Some(Key::Equal),
    RdevKey::LeftBracket => Some(Key::LeftBracket),
    RdevKey::RightBracket => Some(Key::RightBracket),
    RdevKey::BackSlash => Some(Key::BackSlash),
    RdevKey::SemiColon => Some(Key::Semicolon),
    RdevKey::Quote => Some(Key::Quote),
    RdevKey::Comma => Some(Key::Comma),
    RdevKey::Dot => Some(Key::Dot),
    RdevKey::Slash => Some(Key::Slash),

    RdevKey::Kp0 => Some(Key::Numpad(0)),
    RdevKey::Kp1 => Some(Key::Numpad(1)),
    RdevKey::Kp2 => Some(Key::Numpad(2)),
    RdevKey::Kp3 => Some(Key::Numpad(3)),
    RdevKey::Kp4 => Some(Key::Numpad(4)),
    RdevKey::Kp5 => Some(Key::Numpad(5)),
    RdevKey::Kp6 => Some(Key::Numpad(6)),
    RdevKey::Kp7 => Some(Key::Numpad(7)),
    RdevKey::Kp8 => Some(Key::Numpad(8)),
    RdevKey::Kp9 => Some(Key::Numpad(9)),

    RdevKey::KpPlus => Some(Key::NumpadAdd),
    RdevKey::KpMinus => Some(Key::NumpadSubtract),
    RdevKey::KpMultiply => Some(Key::NumpadMultiply),
    RdevKey::KpDivide => Some(Key::NumpadDivide),
    RdevKey::KpReturn => Some(Key::NumpadEnter),
    RdevKey::KpDelete => Some(Key::NumpadDelete),

    RdevKey::Unknown(code) => Some(Key::Unknown(code)),
    _ => None,
  }
}

fn key_event_from_rdev(event: Event) -> Option<KeyEvent> {
  match event.event_type {
    EventType::KeyPress(key) => key_from_rdev(key).map(|key| KeyEvent {
      key,
      kind: KeyEventKind::Press,
    }),
    EventType::KeyRelease(key) => key_from_rdev(key).map(|key| KeyEvent {
      key,
      kind: KeyEventKind::Release,
    }),
    _ => None,
  }
}
