use std::collections::HashSet;
use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};
use std::thread;
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};

use crossterm::event::{
  self as ct_event, Event as CtEvent, MouseEvent as CtMouseEvent,
  MouseEventKind as CtMouseEventKind,
};
use rdev::{listen, Event, EventType, Key as RdevKey};

use super::events::{
  FocusEvent, MouseButton, MouseEvent, MouseEventKind, ResizeEvent, ScrollDirection, SystemEvent,
};

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
  system_sender: Sender<SystemEvent>,
  system_receiver: Receiver<SystemEvent>,
  key_listener_started: Arc<AtomicBool>,
  system_listener_started: Arc<AtomicBool>,
  held_keys: HashSet<Key>,
  pressed_keys: HashSet<Key>,
  released_keys: HashSet<Key>,
  mouse_held_buttons: HashSet<MouseButton>,
  mouse_x: u16,
  mouse_y: u16,
  focused: bool,
  bindings: Vec<KeyBinding>,
}

impl InputService {
  pub fn new() -> Self {
    let (sender, receiver) = unbounded();
    let (action_sender, action_receiver) = unbounded();
    let (system_sender, system_receiver) = unbounded();

    Self {
      sender,
      receiver,
      action_sender,
      action_receiver,
      system_sender,
      system_receiver,
      key_listener_started: Arc::new(AtomicBool::new(false)),
      system_listener_started: Arc::new(AtomicBool::new(false)),
      held_keys: HashSet::new(),
      pressed_keys: HashSet::new(),
      released_keys: HashSet::new(),
      mouse_held_buttons: HashSet::new(),
      mouse_x: 0,
      mouse_y: 0,
      focused: true,
      bindings: Vec::new(),
    }
  }

  pub fn start_key_listener(&self) {
    if self.key_listener_started.swap(true, Ordering::SeqCst) {
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

  /// 运行时 stdin 所有者。
  ///
  /// 此线程通过 crossterm 消费终端事件（resize / focus / mouse），
  /// 阻止键盘字节泄漏回终端。按键语义仍由 rdev 提供。
  /// 启动后任何其他模块不得直接读取 `io::stdin()`。
  pub fn start_system_listener(&self) {
    if self.system_listener_started.swap(true, Ordering::SeqCst) {
      return;
    }

    let sender = self.system_sender.clone();

    thread::spawn(move || {
      // 长轮询间隔，避免空转
      let poll_interval = Duration::from_millis(50);
      loop {
        if ct_event::poll(poll_interval).unwrap_or(false) {
          if let Ok(ct_event) = ct_event::read() {
            match ct_event {
              CtEvent::Key(_) => {}
              other_event => {
                if let Some(sys_event) = system_event_from_crossterm(other_event) {
                  let _ = sender.send(sys_event);
                }
              }
            }
          }
        }
      }
    });
  }

  /// 消费所有待处理的系统事件，每帧调用一次。
  pub fn poll_system_events(&mut self) {
    while let Ok(event) = self.system_receiver.try_recv() {
      self.apply_system_event(&event);
    }
  }

  /// 消费系统事件中的 Resize 事件，用回调更新画布尺寸并标记重绘。
  /// 其余事件（Mouse 等）留到 `drain_system_events` 中处理。
  pub fn poll_resize_events(&mut self, mut on_resize: impl FnMut(u16, u16)) {
    // drain 出所有事件，只处理 Resize，其他的塞回...
    // 但跨线程 channel 不支持 peek/unget。
    // 改为在 drain 时一次性处理 resize。
    let mut others = Vec::new();
    while let Ok(event) = self.system_receiver.try_recv() {
      match &event {
        SystemEvent::Resize(re) => {
          self.apply_system_event(&event);
          on_resize(re.width, re.height);
        }
        _ => others.push(event),
      }
    }
    // 非 resize 事件放回 channel
    for event in others {
      let _ = self.system_sender.send(event);
    }
  }

  /// 获取所有系统事件并合成 Hold 事件（每帧调用一次）。
  ///
  /// 对于当前处于按下状态、但本帧没有 Press/Drag 事件的按钮，
  /// 在事件列表末尾追加一个 Hold 事件（使用最后已知的鼠标坐标）。
  pub fn drain_system_events(&mut self) -> Vec<SystemEvent> {
    let mut events = Vec::new();
    let mut active_buttons: HashSet<MouseButton> = HashSet::new();

    while let Ok(event) = self.system_receiver.try_recv() {
      // 记录本帧有 Press / Drag 的按钮
      if let SystemEvent::Mouse(me) = &event {
        if let Some(button) = me.button {
          match me.kind {
            MouseEventKind::Press | MouseEventKind::Drag => {
              active_buttons.insert(button);
            }
            _ => {}
          }
        }
      }
      self.apply_system_event(&event);
      events.push(event);
    }

    // 合成 Hold 事件：按钮处于按下状态，但本帧没有活跃事件
    for button in &self.mouse_held_buttons {
      if !active_buttons.contains(button) {
        events.push(SystemEvent::Mouse(MouseEvent {
          kind: MouseEventKind::Hold,
          button: Some(*button),
          scroll: None,
          x: self.mouse_x,
          y: self.mouse_y,
        }));
      }
    }

    events
  }

  fn apply_system_event(&mut self, event: &SystemEvent) {
    match event {
      SystemEvent::Focus(focus) => {
        self.focused = focus.gained;
        if !focus.gained {
          // 失焦时清空按键状态，防止按键卡住
          self.held_keys.clear();
          self.pressed_keys.clear();
          self.released_keys.clear();
          self.mouse_held_buttons.clear();
        }
      }
      SystemEvent::Mouse(me) => {
        self.mouse_x = me.x;
        self.mouse_y = me.y;

        if let Some(button) = me.button {
          match me.kind {
            MouseEventKind::Press => {
              self.mouse_held_buttons.insert(button);
            }
            MouseEventKind::Release => {
              self.mouse_held_buttons.remove(&button);
            }
            _ => {}
          }
        }
      }
      _ => {}
    }
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
    // 失焦时拦截所有按键
    if !self.focused {
      return;
    }

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

// ── crossterm 系统事件转换 ──

fn system_event_from_crossterm(event: CtEvent) -> Option<SystemEvent> {
  match event {
    CtEvent::Resize(width, height) => Some(SystemEvent::Resize(ResizeEvent { width, height })),
    CtEvent::FocusGained => Some(SystemEvent::Focus(FocusEvent { gained: true })),
    CtEvent::FocusLost => Some(SystemEvent::Focus(FocusEvent { gained: false })),
    CtEvent::Mouse(me) => Some(SystemEvent::Mouse(mouse_event_from_crossterm(me))),
    _ => None,
  }
}

fn mouse_event_from_crossterm(me: CtMouseEvent) -> MouseEvent {
  let (kind, button, scroll) = match me.kind {
    CtMouseEventKind::Down(btn) => (
      MouseEventKind::Press,
      Some(mouse_button_from_crossterm(btn)),
      None,
    ),
    CtMouseEventKind::Up(btn) => (
      MouseEventKind::Release,
      Some(mouse_button_from_crossterm(btn)),
      None,
    ),
    CtMouseEventKind::Drag(btn) => (
      MouseEventKind::Drag,
      Some(mouse_button_from_crossterm(btn)),
      None,
    ),
    CtMouseEventKind::Moved => (MouseEventKind::Move, None, None),
    CtMouseEventKind::ScrollDown => (MouseEventKind::Scroll, None, Some(ScrollDirection::Down)),
    CtMouseEventKind::ScrollUp => (MouseEventKind::Scroll, None, Some(ScrollDirection::Up)),
    CtMouseEventKind::ScrollLeft => (MouseEventKind::Scroll, None, Some(ScrollDirection::Left)),
    CtMouseEventKind::ScrollRight => (MouseEventKind::Scroll, None, Some(ScrollDirection::Right)),
  };

  MouseEvent {
    kind,
    button,
    scroll,
    x: me.column,
    y: me.row,
  }
}

fn mouse_button_from_crossterm(button: crossterm::event::MouseButton) -> MouseButton {
  match button {
    crossterm::event::MouseButton::Left => MouseButton::Left,
    crossterm::event::MouseButton::Middle => MouseButton::Middle,
    crossterm::event::MouseButton::Right => MouseButton::Right,
  }
}
