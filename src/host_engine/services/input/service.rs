use std::collections::{HashSet, VecDeque};
use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, unbounded};

use crossterm::event::{
  self as ct_event, Event as CtEvent, KeyCode as CtKeyCode, KeyEvent as CtKeyEvent,
  KeyEventKind as CtKeyEventKind, KeyModifiers as CtKeyModifiers, MouseEvent as CtMouseEvent,
  MouseEventKind as CtMouseEventKind,
};
use rdev::{Event, EventType, Key as RdevKey, listen};

use super::events::{
  FocusEvent, MouseButton, MouseEvent, MouseEventKind, ResizeEvent, ScrollDirection, SystemEvent,
  TerminalKeyCode, TerminalKeyEvent,
};
use super::key_token::display_key_token;

/// 键盘按键枚举
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

/// 按键事件类型（按下 / 释放）
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

/// 原始按键事件（含可读显示文本）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawKeyEvent {
  pub key: Key,
  pub display: String,
  pub kind: KeyEventKind,
}

/// 按键状态（按下 / 按住 / 释放）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyState {
  Pressed,
  Held,
  Released,
}

/// 按键模式（单键或双键组合）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyPattern {
  Single(Key),
  Combo(Key, Key),
}

impl KeyPattern {

  /// 将键位规范化排序，使组合键的匹配与按键顺序无关
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

/// 按键绑定（按键模式到动作的映射）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyBinding {
  pub pattern: KeyPattern,
  pub action: String,
}

/// 输入事件类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEventType {
  Keyboard,
}

/// 输入动作事件
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputActionEvent {
  pub event_type: InputEventType,
  pub action: String,
  pub state: KeyState,
}

/// 输入服务，管理键盘/鼠标/系统事件的采集与动作分发
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
  mouse_position: Option<(u16, u16)>,
  focused: bool,
  bindings: Vec<KeyBinding>,
  raw_key_capture_enabled: bool,
  raw_key_events: VecDeque<RawKeyEvent>,
  raw_mouse_capture_enabled: bool,
  raw_mouse_events: VecDeque<MouseEvent>,
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
      mouse_position: None,
      focused: true,
      bindings: Vec::new(),
      raw_key_capture_enabled: false,
      raw_key_events: VecDeque::new(),
      raw_mouse_capture_enabled: false,
      raw_mouse_events: VecDeque::new(),
    }
  }

  /// 启动全局键盘监听线程（仅首次调用生效）
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

  /// 启动系统事件监听线程（终端按键/鼠标/窗口大小/焦点）
  pub fn start_system_listener(&self) {
    if self.system_listener_started.swap(true, Ordering::SeqCst) {
      return;
    }

    let sender = self.system_sender.clone();

    thread::spawn(move || {
      let poll_interval = Duration::from_millis(50);
      loop {
        if ct_event::poll(poll_interval).unwrap_or(false) {
          if let Ok(ct_event) = ct_event::read() {
            match ct_event {
              CtEvent::Key(key_event) => {
                if let Some(event) = terminal_key_event_from_crossterm(key_event) {
                  let _ = sender.send(event);
                }
              }
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

  /// 轮询并应用系统事件队列
  pub fn poll_system_events(&mut self) {
    while let Ok(event) = self.system_receiver.try_recv() {
      self.apply_system_event(&event);
    }
  }

  /// 轮询系统事件并优先处理窗口大小变化
  pub fn poll_resize_events(&mut self, mut on_resize: impl FnMut(u16, u16)) {
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

    for event in others {
      let _ = self.system_sender.send(event);
    }
  }

  /// 排空系统事件队列并返回，同时补齐鼠标 Hold 事件
  pub fn drain_system_events(&mut self) -> Vec<SystemEvent> {
    let mut events = Vec::new();
    let mut active_buttons: HashSet<MouseButton> = HashSet::new();

    while let Ok(event) = self.system_receiver.try_recv() {
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
      if self.focused
        && self.raw_mouse_capture_enabled
        && let SystemEvent::Mouse(mouse) = &event
      {
        self.raw_mouse_events.push_back(*mouse);
      }
      self.apply_system_event(&event);
      events.push(event);
    }

    for button in &self.mouse_held_buttons {
      if !active_buttons.contains(button)
        && let Some((x, y)) = self.mouse_position
      {
        let hold = MouseEvent {
          kind: MouseEventKind::Hold,
          button: Some(*button),
          scroll: None,
          x,
          y,
        };
        if self.focused && self.raw_mouse_capture_enabled {
          self.raw_mouse_events.push_back(hold);
        }
        events.push(SystemEvent::Mouse(hold));
      }
    }

    events
  }

  fn apply_system_event(&mut self, event: &SystemEvent) {
    match event {
      SystemEvent::Focus(focus) => {
        self.focused = focus.gained;
        if !focus.gained {
          self.held_keys.clear();
          self.pressed_keys.clear();
          self.released_keys.clear();
          self.mouse_held_buttons.clear();
        }
      }
      SystemEvent::Mouse(me) => {
        self.mouse_position = Some((me.x, me.y));

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

  /// 开始新的一帧，清空单帧按键状态
  pub fn begin_frame(&mut self) {
    self.pressed_keys.clear();
    self.released_keys.clear();
  }

  /// 轮询并应用全局键盘事件
  pub fn poll(&mut self) {
    while let Ok(event) = self.receiver.try_recv() {
      if self.focused && self.raw_key_capture_enabled {
        self.raw_key_events.push_back(RawKeyEvent {
          key: event.key,
          display: display_key_token(event.key),
          kind: event.kind,
        });
      }
      self.apply_key_event(event);
    }
  }

  /// 启用原始按键捕获
  pub fn enable_raw_key_capture(&mut self) -> bool {
    if self.raw_key_capture_enabled {
      return false;
    }
    self.raw_key_events.clear();
    self.raw_key_capture_enabled = true;
    true
  }

  /// 禁用原始按键捕获
  pub fn disable_raw_key_capture(&mut self) -> bool {
    if !self.raw_key_capture_enabled {
      return false;
    }
    self.raw_key_capture_enabled = false;
    true
  }

  pub fn is_raw_key_capture_enabled(&self) -> bool {
    self.raw_key_capture_enabled
  }

  /// 取出所有原始按键事件
  pub fn take_raw_key_events(&mut self) -> Vec<RawKeyEvent> {
    self.raw_key_events.drain(..).collect()
  }

  /// 启用原始鼠标事件捕获
  pub fn enable_raw_mouse_capture(&mut self) -> bool {
    if self.raw_mouse_capture_enabled {
      return false;
    }
    self.raw_mouse_events.clear();
    self.raw_mouse_capture_enabled = true;
    true
  }

  /// 禁用原始鼠标事件捕获
  pub fn disable_raw_mouse_capture(&mut self) -> bool {
    if !self.raw_mouse_capture_enabled {
      return false;
    }
    self.raw_mouse_capture_enabled = false;
    true
  }

  pub fn is_raw_mouse_capture_enabled(&self) -> bool {
    self.raw_mouse_capture_enabled
  }

  /// 取出所有原始鼠标事件
  pub fn take_raw_mouse_events(&mut self) -> Vec<MouseEvent> {
    self.raw_mouse_events.drain(..).collect()
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

  /// 查询按键在当前帧的状态
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

  pub fn mouse_position(&self) -> Option<(u16, u16)> {
    self.mouse_position
  }

  pub fn is_mouse_down(&self, button: MouseButton) -> bool {
    self.mouse_held_buttons.contains(&button)
  }

  /// 清空所有按键状态
  pub fn clear(&mut self) {
    self.held_keys.clear();
    self.pressed_keys.clear();
    self.released_keys.clear();
  }

  /// 加载按键绑定配置
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

  // 组合键状态判定：任意键释放即认为组合键释放，后按的键触发按下
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

  /// 根据当前按键状态和绑定表收集动作事件
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

  /// 收集动作事件并发送到动作通道
  pub fn dispatch_action_events(&self) {
    for event in self.collect_action_events() {
      let _ = self.action_sender.send(event);
    }
  }

  /// 获取下一个动作事件
  pub fn next_action_event(&self) -> Option<InputActionEvent> {
    self.action_receiver.try_recv().ok()
  }

  fn apply_key_event(&mut self, event: KeyEvent) {
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

// 将 rdev 按键映射为内部 Key 枚举
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

// 将 crossterm 按键事件转换为终端按键系统事件，过滤释放/修饰键组合
fn terminal_key_event_from_crossterm(event: CtKeyEvent) -> Option<SystemEvent> {
  if event.kind == CtKeyEventKind::Release {
    return None;
  }

  let ctrl = event.modifiers.contains(CtKeyModifiers::CONTROL);
  let shift = event.modifiers.contains(CtKeyModifiers::SHIFT);
  let rejected =
    CtKeyModifiers::ALT | CtKeyModifiers::SUPER | CtKeyModifiers::HYPER | CtKeyModifiers::META;
  if event.modifiers.intersects(rejected) {
    return None;
  }
  let allowed_modifiers = match event.code {
    CtKeyCode::Char(ch) if ctrl => "acxv".contains(ch.to_ascii_lowercase()) && !shift,
    CtKeyCode::Char(_) => !ctrl,
    CtKeyCode::Enter => !shift,
    CtKeyCode::Left | CtKeyCode::Right => true,
    CtKeyCode::Up | CtKeyCode::Down | CtKeyCode::Home | CtKeyCode::End => !ctrl,
    _ => !ctrl && !shift,
  };
  if !allowed_modifiers {
    return None;
  }

  let code = match event.code {
    CtKeyCode::Char(ch) if !ch.is_control() => TerminalKeyCode::Char(ch),
    CtKeyCode::Enter => TerminalKeyCode::Enter,
    CtKeyCode::Esc => TerminalKeyCode::Esc,
    CtKeyCode::Backspace => TerminalKeyCode::Backspace,
    CtKeyCode::Delete => TerminalKeyCode::Delete,
    CtKeyCode::Left => TerminalKeyCode::Left,
    CtKeyCode::Right => TerminalKeyCode::Right,
    CtKeyCode::Up => TerminalKeyCode::Up,
    CtKeyCode::Down => TerminalKeyCode::Down,
    CtKeyCode::Home => TerminalKeyCode::Home,
    CtKeyCode::End => TerminalKeyCode::End,
    _ => return None,
  };

  Some(SystemEvent::TerminalKey(TerminalKeyEvent {
    code,
    ctrl,
    shift,
  }))
}

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

#[cfg(test)]
mod tests {
  use super::*;

  fn key(code: CtKeyCode) -> CtKeyEvent {
    CtKeyEvent::new(code, CtKeyModifiers::NONE)
  }

  fn terminal_code(event: CtKeyEvent) -> Option<TerminalKeyCode> {
    match terminal_key_event_from_crossterm(event) {
      Some(SystemEvent::TerminalKey(event)) => Some(event.code),
      _ => None,
    }
  }

  fn terminal_event(event: CtKeyEvent) -> Option<TerminalKeyEvent> {
    match terminal_key_event_from_crossterm(event) {
      Some(SystemEvent::TerminalKey(event)) => Some(event),
      _ => None,
    }
  }

  #[test]
  fn raw_key_capture_runs_alongside_action_map() {
    let mut input = InputService::new();
    input.load_key_bindings(vec![KeyBinding {
      pattern: KeyPattern::Single(Key::A),
      action: "test.a".to_string(),
    }]);
    assert!(input.enable_raw_key_capture());
    assert!(!input.enable_raw_key_capture());

    input
      .sender
      .send(KeyEvent {
        key: Key::A,
        kind: KeyEventKind::Press,
      })
      .unwrap();
    input.poll();

    assert_eq!(
      input.take_raw_key_events(),
      vec![RawKeyEvent {
        key: Key::A,
        display: "A".to_string(),
        kind: KeyEventKind::Press,
      }]
    );
    assert_eq!(input.collect_action_events()[0].action, "test.a");
    assert_eq!(input.collect_action_events()[0].state, KeyState::Pressed);
  }

  #[test]
  fn raw_key_capture_disable_preserves_events_and_enable_starts_clean() {
    let mut input = InputService::new();
    input.enable_raw_key_capture();
    input
      .sender
      .send(KeyEvent {
        key: Key::Left,
        kind: KeyEventKind::Press,
      })
      .unwrap();
    input.poll();
    assert!(input.disable_raw_key_capture());
    assert!(!input.disable_raw_key_capture());
    assert_eq!(input.take_raw_key_events()[0].display, "←");

    input.enable_raw_key_capture();
    input
      .sender
      .send(KeyEvent {
        key: Key::Left,
        kind: KeyEventKind::Release,
      })
      .unwrap();
    input.poll();
    assert_eq!(input.take_raw_key_events()[0].kind, KeyEventKind::Release);

    input
      .sender
      .send(KeyEvent {
        key: Key::B,
        kind: KeyEventKind::Press,
      })
      .unwrap();
    input.poll();
    input.disable_raw_key_capture();
    input.enable_raw_key_capture();
    assert!(input.take_raw_key_events().is_empty());
  }

  #[test]
  fn action_map_still_reports_pressed_held_and_released() {
    let mut input = InputService::new();
    input.load_key_bindings(vec![KeyBinding {
      pattern: KeyPattern::Single(Key::A),
      action: "test.a".to_string(),
    }]);
    input.apply_key_event(KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Press,
    });
    assert_eq!(input.collect_action_events()[0].state, KeyState::Pressed);
    input.begin_frame();
    assert_eq!(input.collect_action_events()[0].state, KeyState::Held);
    input.apply_key_event(KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Release,
    });
    assert_eq!(input.collect_action_events()[0].state, KeyState::Released);
  }

  #[test]
  fn mouse_queries_track_position_buttons_and_focus_loss() {
    let mut input = InputService::new();
    assert_eq!(input.mouse_position(), None);
    assert!(!input.is_mouse_down(MouseButton::Middle));
    input.apply_system_event(&SystemEvent::Mouse(MouseEvent {
      kind: MouseEventKind::Press,
      button: Some(MouseButton::Middle),
      scroll: None,
      x: 7,
      y: 9,
    }));
    assert_eq!(input.mouse_position(), Some((7, 9)));
    assert!(input.is_mouse_down(MouseButton::Middle));
    input.apply_system_event(&SystemEvent::Focus(FocusEvent { gained: false }));
    assert!(!input.is_mouse_down(MouseButton::Middle));
    assert_eq!(input.mouse_position(), Some((7, 9)));
  }

  #[test]
  fn terminal_key_event_from_crossterm_maps_supported_keys() {
    let cases = [
      (CtKeyCode::Char('a'), TerminalKeyCode::Char('a')),
      (CtKeyCode::Char('我'), TerminalKeyCode::Char('我')),
      (CtKeyCode::Enter, TerminalKeyCode::Enter),
      (CtKeyCode::Esc, TerminalKeyCode::Esc),
      (CtKeyCode::Backspace, TerminalKeyCode::Backspace),
      (CtKeyCode::Delete, TerminalKeyCode::Delete),
      (CtKeyCode::Left, TerminalKeyCode::Left),
      (CtKeyCode::Right, TerminalKeyCode::Right),
      (CtKeyCode::Up, TerminalKeyCode::Up),
      (CtKeyCode::Down, TerminalKeyCode::Down),
      (CtKeyCode::Home, TerminalKeyCode::Home),
      (CtKeyCode::End, TerminalKeyCode::End),
    ];

    for (input, expected) in cases {
      assert_eq!(terminal_code(key(input)), Some(expected));
    }
    assert_eq!(terminal_code(key(CtKeyCode::F(1))), None);
    assert_eq!(terminal_code(key(CtKeyCode::Tab)), None);
  }

  #[test]
  fn terminal_key_event_from_crossterm_filters_kind_and_modifiers() {
    assert_eq!(
      terminal_code(CtKeyEvent::new_with_kind(
        CtKeyCode::Char('a'),
        CtKeyModifiers::NONE,
        CtKeyEventKind::Repeat,
      )),
      Some(TerminalKeyCode::Char('a'))
    );
    assert_eq!(
      terminal_code(CtKeyEvent::new_with_kind(
        CtKeyCode::Char('a'),
        CtKeyModifiers::NONE,
        CtKeyEventKind::Release,
      )),
      None
    );
    assert_eq!(
      terminal_code(CtKeyEvent::new(CtKeyCode::Char('A'), CtKeyModifiers::SHIFT,)),
      Some(TerminalKeyCode::Char('A'))
    );
    assert_eq!(
      terminal_event(CtKeyEvent::new(CtKeyCode::Enter, CtKeyModifiers::CONTROL)),
      Some(TerminalKeyEvent {
        code: TerminalKeyCode::Enter,
        ctrl: true,
        shift: false,
      })
    );
    assert_eq!(
      terminal_event(CtKeyEvent::new(
        CtKeyCode::Char('a'),
        CtKeyModifiers::CONTROL,
      )),
      Some(TerminalKeyEvent {
        code: TerminalKeyCode::Char('a'),
        ctrl: true,
        shift: false,
      })
    );
    assert_eq!(
      terminal_code(CtKeyEvent::new(
        CtKeyCode::Left,
        CtKeyModifiers::CONTROL | CtKeyModifiers::SHIFT,
      )),
      Some(TerminalKeyCode::Left)
    );
    assert_eq!(
      terminal_code(CtKeyEvent::new(
        CtKeyCode::Char('z'),
        CtKeyModifiers::CONTROL,
      )),
      None
    );
    for modifiers in [
      CtKeyModifiers::ALT,
      CtKeyModifiers::SUPER,
      CtKeyModifiers::HYPER,
      CtKeyModifiers::META,
    ] {
      assert_eq!(
        terminal_code(CtKeyEvent::new(CtKeyCode::Char('a'), modifiers)),
        None
      );
    }
  }

  #[test]
  fn terminal_key_survives_resize_poll() {
    let mut input = InputService::new();
    let a = SystemEvent::TerminalKey(TerminalKeyEvent {
      code: TerminalKeyCode::Char('a'),
      ctrl: false,
      shift: false,
    });
    let b = SystemEvent::TerminalKey(TerminalKeyEvent {
      code: TerminalKeyCode::Char('b'),
      ctrl: false,
      shift: false,
    });
    input.system_sender.send(a).unwrap();
    input
      .system_sender
      .send(SystemEvent::Resize(ResizeEvent {
        width: 100,
        height: 30,
      }))
      .unwrap();
    input.system_sender.send(b).unwrap();

    let mut resize = None;
    input.poll_resize_events(|width, height| resize = Some((width, height)));

    assert_eq!(resize, Some((100, 30)));
    assert_eq!(input.drain_system_events(), vec![a, b]);
  }

  #[test]
  fn raw_mouse_capture_runs_alongside_system_events() {
    let mut input = InputService::new();
    assert!(!input.is_raw_mouse_capture_enabled());
    assert!(input.enable_raw_mouse_capture());
    assert!(!input.enable_raw_mouse_capture());

    let move_evt = SystemEvent::Mouse(MouseEvent {
      kind: MouseEventKind::Move,
      button: None,
      scroll: None,
      x: 10,
      y: 5,
    });
    let scroll_evt = SystemEvent::Mouse(MouseEvent {
      kind: MouseEventKind::Scroll,
      button: None,
      scroll: Some(ScrollDirection::Down),
      x: 20,
      y: 10,
    });
    input.system_sender.send(move_evt).unwrap();
    input.system_sender.send(scroll_evt).unwrap();

    let system_events = input.drain_system_events();
    assert_eq!(system_events, vec![move_evt, scroll_evt]);

    let raw_mouse = input.take_raw_mouse_events();
    assert_eq!(raw_mouse.len(), 2);
    assert!(matches!(raw_mouse[0].kind, MouseEventKind::Move));
    assert!(matches!(raw_mouse[1].kind, MouseEventKind::Scroll));

    assert!(input.take_raw_mouse_events().is_empty());
  }

  #[test]
  fn raw_mouse_capture_respects_focus() {
    let mut input = InputService::new();
    input.enable_raw_mouse_capture();

    input.focused = false;
    input
      .system_sender
      .send(SystemEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Move,
        button: None,
        scroll: None,
        x: 1,
        y: 1,
      }))
      .unwrap();
    input.drain_system_events();
    assert!(input.take_raw_mouse_events().is_empty());

    input.focused = true;
    input
      .system_sender
      .send(SystemEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Move,
        button: None,
        scroll: None,
        x: 2,
        y: 2,
      }))
      .unwrap();
    input.drain_system_events();
    assert_eq!(input.take_raw_mouse_events().len(), 1);
  }

  #[test]
  fn raw_mouse_disable_preserves_events_and_enable_starts_clean() {
    let mut input = InputService::new();
    input.enable_raw_mouse_capture();
    input
      .system_sender
      .send(SystemEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Move,
        button: None,
        scroll: None,
        x: 3,
        y: 3,
      }))
      .unwrap();
    input.drain_system_events();
    assert!(input.disable_raw_mouse_capture());
    assert!(!input.disable_raw_mouse_capture());
    assert_eq!(input.take_raw_mouse_events().len(), 1);

    input.enable_raw_mouse_capture();
    assert!(input.take_raw_mouse_events().is_empty());
  }
}
