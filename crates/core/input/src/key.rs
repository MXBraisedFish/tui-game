//! Keyboard identities, normalized key combinations, frame states, and action events.

/// A portable keyboard key identity independent of the platform input backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Key {
  /// The esc key.
  Esc,

  /// The enter key.
  Enter,
  /// The tab key.
  Tab,
  /// The backspace key.
  Backspace,
  /// The space key.
  Space,

  /// The up key.
  Up,
  /// The down key.
  Down,
  /// The left key.
  Left,
  /// The right key.
  Right,

  /// The home key.
  Home,
  /// The end key.
  End,
  /// The page up key.
  PageUp,
  /// The page down key.
  PageDown,
  /// The insert key.
  Insert,
  /// The delete key.
  Delete,

  /// The fn key.
  Fn(u8),

  /// The num key.
  Num(u8),
  /// The numpad key.
  Numpad(u8),

  /// The A letter key.
  A,
  /// The B letter key.
  B,
  /// The C letter key.
  C,
  /// The D letter key.
  D,
  /// The E letter key.
  E,
  /// The F letter key.
  F,
  /// The G letter key.
  G,
  /// The H letter key.
  H,
  /// The I letter key.
  I,
  /// The J letter key.
  J,
  /// The K letter key.
  K,
  /// The L letter key.
  L,
  /// The M letter key.
  M,
  /// The N letter key.
  N,
  /// The O letter key.
  O,
  /// The P letter key.
  P,
  /// The Q letter key.
  Q,
  /// The R letter key.
  R,
  /// The S letter key.
  S,
  /// The T letter key.
  T,
  /// The U letter key.
  U,
  /// The V letter key.
  V,
  /// The W letter key.
  W,
  /// The X letter key.
  X,
  /// The Y letter key.
  Y,
  /// The Z letter key.
  Z,

  /// The left ctrl key.
  LeftCtrl,
  /// The right ctrl key.
  RightCtrl,
  /// The left shift key.
  LeftShift,
  /// The right shift key.
  RightShift,
  /// The left alt key.
  LeftAlt,
  /// The right alt key.
  RightAlt,
  /// The left meta key.
  LeftMeta,
  /// The right meta key.
  RightMeta,

  /// The caps lock key.
  CapsLock,
  /// The num lock key.
  NumLock,
  /// The scroll lock key.
  ScrollLock,

  /// The print screen key.
  PrintScreen,
  /// The pause key.
  Pause,

  /// The back quote key.
  BackQuote,
  /// The minus key.
  Minus,
  /// The equal key.
  Equal,
  /// The left bracket key.
  LeftBracket,
  /// The right bracket key.
  RightBracket,
  /// The back slash key.
  BackSlash,
  /// The semicolon key.
  Semicolon,
  /// The quote key.
  Quote,
  /// The comma key.
  Comma,
  /// The dot key.
  Dot,
  /// The slash key.
  Slash,

  /// The numpad add key.
  NumpadAdd,
  /// The numpad subtract key.
  NumpadSubtract,
  /// The numpad multiply key.
  NumpadMultiply,
  /// The numpad divide key.
  NumpadDivide,
  /// The numpad enter key.
  NumpadEnter,
  /// The numpad delete key.
  NumpadDelete,

  /// The unknown key.
  Unknown(u32),
}

/// A press or release transition from the global keyboard backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEventKind {
  /// A press notification delivered to the owning consumer.
  Press,
  /// A release notification delivered to the owning consumer.
  Release,
}

/// A portable key paired with its transition kind.
///
/// # Fields
///
/// * `key` - The lookup key.
/// * `kind` - The key event kind carried by this key event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
  /// The lookup key.
  pub key: Key,
  /// The key event kind carried by this key event.
  pub kind: KeyEventKind,
}

/// A raw key transition with display text for shortcut capture.
///
/// # Fields
///
/// * `key` - The lookup key.
/// * `display` - The display.
/// * `kind` - The key event kind carried by this raw key event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawKeyEvent {
  /// The lookup key.
  pub key: Key,
  /// The display.
  pub display: String,
  /// The key event kind carried by this raw key event.
  pub kind: KeyEventKind,
}

/// The pressed, held, or released state observed during a host frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyState {
  /// A transition into the pressed state.
  Pressed,
  /// The operation is held.
  Held,
  /// A transition out of the pressed state.
  Released,
}

/// A single key or normalized two-key shortcut combination.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyPattern {
  /// The single setting for key pattern.
  Single(Key),
  /// The combo setting for key pattern.
  Combo(Key, Key),
}

impl KeyPattern {
  /// Return the key pattern with combination keys sorted into canonical order.
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
}

/// A key pattern associated with an action name.
///
/// # Fields
///
/// * `pattern` - The pattern.
/// * `priority` - The nonnegative dispatch priority; larger values are dispatched first.
/// * `action` - The action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyBinding {
  /// The pattern.
  pub pattern: KeyPattern,
  /// The action.
  pub action: String,
  /// The nonnegative dispatch priority; larger values are dispatched first.
  pub priority: u64,
}

/// The input device category attached to an action event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEventType {
  /// The keyboard setting for input event type.
  Keyboard,
}

/// An action name and input state produced by a matching key binding.
///
/// # Fields
///
/// * `event_type` - The event type.
/// * `action` - The action.
/// * `state` - The key state carried by this input action event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputActionEvent {
  /// The event type.
  pub event_type: InputEventType,
  /// The action.
  pub action: String,
  /// The key state carried by this input action event.
  pub state: KeyState,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn combo_patterns_are_order_independent() {
    let combo = KeyPattern::Combo(Key::Z, Key::A);
    assert_eq!(combo.normalized(), KeyPattern::Combo(Key::A, Key::Z));
    assert_eq!(
      combo.normalized(),
      KeyPattern::Combo(Key::A, Key::Z).normalized()
    );
  }
}
