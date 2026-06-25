use crate::host_engine::services::{Rect, SliceId, TextColor, TextStyle};

/// 文本输入组件的唯一标识符。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextInputId(pub u64);

/// 文本输入模式：单行或多行。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextInputMode {
  #[default]
  SingleLine,
  MultiLine,
}

/// 垂直对齐方式。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VerticalAlign {
  #[default]
  Top,
  Center,
  Bottom,
}

/// 光标形状。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextInputCursorShape {
  #[default]
  Block,
  Underline,
  None,
  Line,
}

/// 文本输入组件创建选项。
#[derive(Clone, Debug, Default)]
pub struct TextInputOptions {
  pub initial_text: String,
  pub max_chars: Option<usize>,
  pub mode: TextInputMode,
  pub mouse: bool,
}

/// 文本输入组件的渲染参数。
#[derive(Clone, Debug)]
pub struct TextInputRenderParams {
  pub rect: Rect,
  pub placeholder: String,
  pub fg: Option<TextColor>,
  pub bg: Option<TextColor>,
  pub placeholder_fg: Option<TextColor>,
  pub text_style: TextStyle,
  pub placeholder_style: TextStyle,
  pub cursor_style: TextStyle,
  pub cursor_shape: Option<TextInputCursorShape>,
  pub cursor_blink: bool,
  pub vertical_align: VerticalAlign,
}

impl Default for TextInputRenderParams {
  fn default() -> Self {
    Self {
      rect: Rect::default(),
      placeholder: String::new(),
      fg: None,
      bg: None,
      placeholder_fg: None,
      text_style: TextStyle::default(),
      placeholder_style: TextStyle::default(),
      cursor_style: TextStyle::default(),
      cursor_shape: None,
      cursor_blink: true,
      vertical_align: VerticalAlign::Top,
    }
  }
}

/// 文本输入组件事件。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextInputEvent {
  Focused { id: TextInputId },
  Blurred { id: TextInputId },
  Changed { id: TextInputId, value: String },
  Submit { id: TextInputId, value: String },
  Cancel { id: TextInputId, value: String },
  Pressed { id: TextInputId },
  PressedOutside { id: TextInputId },
}

#[derive(Clone, Copy)]
pub(super) enum TextSurface {
  Base,
  Slice(SliceId),
  Host,
}
