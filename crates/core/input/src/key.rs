use std::collections::HashSet;

/// Keyboard key.
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

/// Kind of key event (press or release).
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

/// Raw key event, including its human-readable display text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawKeyEvent {
  pub key: Key,
  pub display: String,
  pub kind: KeyEventKind,
}

/// Key state (pressed, held or released).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyState {
  Pressed,
  Held,
  Released,
}

/// Key pattern: a single key or a two-key combination.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyPattern {
  Single(Key),
  Combo(Key, Key),
}

impl KeyPattern {
  /// Returns the pattern with its keys sorted, so combo matching does not depend on key order.
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

/// Binding from a key pattern to an action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyBinding {
  pub pattern: KeyPattern,
  pub action: String,
}

/// Type of input event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEventType {
  Keyboard,
}

/// Input event that reports the state of a bound action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputActionEvent {
  pub event_type: InputEventType,
  pub action: String,
  pub state: KeyState,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn combo_patterns_are_order_independent_and_consume_both_keys() {
    let combo = KeyPattern::Combo(Key::Z, Key::A);
    assert_eq!(combo.normalized(), KeyPattern::Combo(Key::A, Key::Z));
    assert_eq!(combo.normalized(), KeyPattern::Combo(Key::A, Key::Z).normalized());

    let mut consumed = HashSet::new();
    assert!(!combo.has_consumed_key(&consumed));
    combo.consume_keys(&mut consumed);
    assert_eq!(consumed, HashSet::from([Key::A, Key::Z]));
    assert!(KeyPattern::Single(Key::Z).has_consumed_key(&consumed));
    assert!(!KeyPattern::Single(Key::B).has_consumed_key(&consumed));
  }
}
