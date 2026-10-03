//! Portable terminal key, mouse, resize, and focus event payloads.

/// A key code reported by the terminal input backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalKeyCode {
  /// The char key.
  Char(char),
  /// The enter key.
  Enter,
  /// The esc key.
  Esc,
  /// The backspace key.
  Backspace,
  /// The delete key.
  Delete,
  /// The left key.
  Left,
  /// The right key.
  Right,
  /// The up key.
  Up,
  /// The down key.
  Down,
  /// The home key.
  Home,
  /// The end key.
  End,
}

/// A terminal key code and its pressed modifier flags.
///
/// # Fields
///
/// * `code` - The stable error or language code.
/// * `ctrl` - Whether the control modifier is active.
/// * `shift` - Whether the shift modifier is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalKeyEvent {
  /// The stable error or language code.
  pub code: TerminalKeyCode,
  /// Whether the control modifier is active.
  pub ctrl: bool,
  /// Whether the shift modifier is active.
  pub shift: bool,
}

/// The new physical terminal dimensions after a resize.
///
/// # Fields
///
/// * `width` - The width in terminal columns.
/// * `height` - The height in terminal rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResizeEvent {
  /// The width in terminal columns.
  pub width: u16,
  /// The height in terminal rows.
  pub height: u16,
}

/// A terminal focus-gained or focus-lost notification.
///
/// # Fields
///
/// * `gained` - Whether terminal focus was gained rather than lost.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusEvent {
  /// Whether terminal focus was gained rather than lost.
  pub gained: bool,
}

/// The pointer button reported by a terminal mouse event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
  /// The left setting for mouse button.
  Left,
  /// The middle setting for mouse button.
  Middle,
  /// The right setting for mouse button.
  Right,
}

/// The type of pointer transition reported by the terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseEventKind {
  /// A press notification delivered to the owning consumer.
  Press,
  /// A release notification delivered to the owning consumer.
  Release,
  /// A move notification delivered to the owning consumer.
  Move,
  /// A drag notification delivered to the owning consumer.
  Drag,

  /// A held input repeated while it remains active.
  Hold,
  /// A scroll notification delivered to the owning consumer.
  Scroll,
}

/// The direction of a terminal wheel event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollDirection {
  /// The up setting for scroll direction.
  Up,
  /// The down setting for scroll direction.
  Down,
  /// The left setting for scroll direction.
  Left,
  /// The right setting for scroll direction.
  Right,
}

/// A pointer transition, button, and physical terminal-cell position.
///
/// # Fields
///
/// * `kind` - The mouse event kind carried by this mouse event.
/// * `button` - The mouse button to query.
/// * `scroll` - The scroll.
/// * `x` - The horizontal coordinate in terminal cells.
/// * `y` - The vertical coordinate in terminal cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseEvent {
  /// The mouse event kind carried by this mouse event.
  pub kind: MouseEventKind,
  /// The mouse button to query.
  pub button: Option<MouseButton>,
  /// The scroll.
  pub scroll: Option<ScrollDirection>,
  /// The horizontal coordinate in terminal cells.
  pub x: u16,
  /// The vertical coordinate in terminal cells.
  pub y: u16,
}

/// A terminal key, pointer, resize, or focus event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemEvent {
  /// A resize notification delivered to the owning consumer.
  Resize(ResizeEvent),
  /// A focus notification delivered to the owning consumer.
  Focus(FocusEvent),
  /// A mouse notification delivered to the owning consumer.
  Mouse(MouseEvent),
  /// A terminal key notification delivered to the owning consumer.
  TerminalKey(TerminalKeyEvent),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn terminal_key_event_can_be_constructed() {
    let event = SystemEvent::TerminalKey(TerminalKeyEvent {
      code: TerminalKeyCode::Char('我'),
      ctrl: false,
      shift: false,
    });

    assert_eq!(
      event,
      SystemEvent::TerminalKey(TerminalKeyEvent {
        code: TerminalKeyCode::Char('我'),
        ctrl: false,
        shift: false,
      })
    );
  }
}
