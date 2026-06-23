// ── 系统事件 ──

/// 文本输入关心的终端按键。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalKeyCode {
  Char(char),
  Enter,
  Esc,
  Backspace,
  Delete,
  Left,
  Right,
  Home,
  End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalKeyEvent {
  pub code: TerminalKeyCode,
}

/// 终端尺寸变化事件。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResizeEvent {
  pub width: u16,
  pub height: u16,
}

/// 焦点变化事件。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusEvent {
  /// true = 获得焦点，false = 失去焦点
  pub gained: bool,
}

/// 鼠标按键。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
  Left,
  Middle,
  Right,
}

/// 鼠标事件类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseEventKind {
  /// 按下
  Press,
  /// 松开
  Release,
  /// 移动（无按键）
  Move,
  /// 拖动（按住按键移动）
  Drag,
  /// 持续按住
  Hold,
  /// 滚动
  Scroll,
}

/// 滚动方向。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollDirection {
  Up,
  Down,
  Left,
  Right,
}

/// 鼠标事件。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseEvent {
  pub kind: MouseEventKind,
  /// Move / Scroll 事件不携带按键信息，此时为 `None`
  pub button: Option<MouseButton>,
  /// 仅滚动事件时有值
  pub scroll: Option<ScrollDirection>,
  pub x: u16,
  pub y: u16,
}

/// 由终端直接提供的系统事件。
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
    });

    assert_eq!(
      event,
      SystemEvent::TerminalKey(TerminalKeyEvent {
        code: TerminalKeyCode::Char('我'),
      })
    );
  }
}
