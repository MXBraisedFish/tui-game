
/// 终端按键码
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalKeyCode {
  Char(char),
  Enter,
  Esc,
  Backspace,
  Delete,
  Left,
  Right,
  Up,
  Down,
  Home,
  End,
}

/// 终端按键事件（含修饰键信息）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalKeyEvent {
  pub code: TerminalKeyCode,
  pub ctrl: bool,
  pub shift: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResizeEvent {
  pub width: u16,
  pub height: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusEvent {
  pub gained: bool,
}

/// 鼠标按键
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
  Left,
  Middle,
  Right,
}

/// 鼠标事件类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseEventKind {
  Press,
  Release,
  Move,
  Drag,

  Hold,
  Scroll,
}

/// 滚轮方向
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollDirection {
  Up,
  Down,
  Left,
  Right,
}

/// 鼠标事件
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseEvent {
  pub kind: MouseEventKind,
  pub button: Option<MouseButton>,
  pub scroll: Option<ScrollDirection>,
  pub x: u16,
  pub y: u16,
}

/// 系统事件（终端按键 / 鼠标 / 窗口大小 / 焦点）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemEvent {
  Resize(ResizeEvent),
  Focus(FocusEvent),
  Mouse(MouseEvent),
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
