//! Identifiers, configuration, states, and events shared by this module.

pub use tg_service_text_layout::TextAlign;

use crate::SliceId;
use tg_core_style::{TextColor, TextStyle};
use tg_service_layout::Rect;

/// The identity of text input within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextInputId(
  /// The wrapped u64 value.
  pub u64,
);

/// The single-line or multi-line editing behavior of a text input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextInputMode {
  /// The single line setting for text input mode.
  #[default]
  SingleLine,
  /// The multi line setting for text input mode.
  MultiLine,
}

/// Vertical placement of content within the available input height.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VerticalAlign {
  /// The top setting for vertical align.
  #[default]
  Top,
  /// The center setting for vertical align.
  Center,
  /// The bottom setting for vertical align.
  Bottom,
}

/// The terminal cursor shape requested by the focused text input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextInputCursorShape {
  /// The block setting for text input cursor shape.
  #[default]
  Block,
  /// The underline setting for text input cursor shape.
  Underline,
  /// The none setting for text input cursor shape.
  None,
  /// The line setting for text input cursor shape.
  Line,
}

/// Configuration values controlling text input behavior.
///
/// # Fields
///
/// * `initial_text` - The text loaded into a newly created input.
/// * `max_chars` - The optional character-count limit enforced when editing.
/// * `mode` - The text input mode carried by this text input options.
/// * `mouse` - Whether pointer input may focus, position the cursor, or select text.
#[derive(Clone, Debug, Default)]
pub struct TextInputOptions {
  /// The text loaded into a newly created input.
  pub initial_text: String,
  /// The optional character-count limit enforced when editing.
  pub max_chars: Option<usize>,
  /// The text input mode carried by this text input options.
  pub mode: TextInputMode,
  /// Whether pointer input may focus, position the cursor, or select text.
  pub mouse: bool,
}

/// Configuration values controlling text input render behavior.
///
/// # Fields
///
/// * `rect` - The rectangular region in terminal cells.
/// * `placeholder` - The placeholder.
/// * `fg` - The foreground color override, or `None` to inherit the default.
/// * `bg` - The background color override, or `None` to inherit the default.
/// * `placeholder_fg` - The placeholder fg.
/// * `text_style` - The text style.
/// * `placeholder_style` - The placeholder style.
/// * `cursor_style` - The cursor style.
/// * `cursor_shape` - The cursor shape.
/// * `cursor_blink` - The cursor blink.
/// * `vertical_align` - The vertical align.
/// * `text_align` - The text align.
#[derive(Clone, Debug)]
pub struct TextInputRenderParams {
  /// The rectangular region in terminal cells.
  pub rect: Rect,
  /// The placeholder.
  pub placeholder: String,
  /// The foreground color override, or `None` to inherit the default.
  pub fg: Option<TextColor>,
  /// The background color override, or `None` to inherit the default.
  pub bg: Option<TextColor>,
  /// The placeholder fg.
  pub placeholder_fg: Option<TextColor>,
  /// The text style.
  pub text_style: TextStyle,
  /// The placeholder style.
  pub placeholder_style: TextStyle,
  /// The cursor style.
  pub cursor_style: TextStyle,
  /// The cursor shape.
  pub cursor_shape: Option<TextInputCursorShape>,
  /// The cursor blink.
  pub cursor_blink: bool,
  /// The vertical align.
  pub vertical_align: VerticalAlign,
  /// The text align.
  pub text_align: TextAlign,
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
      text_align: TextAlign::Left,
    }
  }
}

/// A text input event payload queued for its owning consumer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextInputEvent {
  /// A focused notification delivered to the owning consumer.
  Focused {
    /// The identifier of the owned object.
    id: TextInputId,
  },
  /// A blurred notification delivered to the owning consumer.
  Blurred {
    /// The identifier of the owned object.
    id: TextInputId,
  },
  /// A changed notification delivered to the owning consumer.
  Changed {
    /// The identifier of the owned object.
    id: TextInputId,
    /// The value to store or convert.
    value: String,
  },
  /// A submit notification delivered to the owning consumer.
  Submit {
    /// The identifier of the owned object.
    id: TextInputId,
    /// The value to store or convert.
    value: String,
  },
  /// A cancel notification delivered to the owning consumer.
  Cancel {
    /// The identifier of the owned object.
    id: TextInputId,
    /// The value to store or convert.
    value: String,
  },
  /// A transition into the pressed state.
  Pressed {
    /// The identifier of the owned object.
    id: TextInputId,
  },
  /// A pressed outside notification delivered to the owning consumer.
  PressedOutside {
    /// The identifier of the owned object.
    id: TextInputId,
  },
}

/// The drawing surface used to resolve text-input position and clipping.
#[derive(Clone, Copy)]
pub(super) enum TextSurface {
  /// The base setting for text surface.
  Base,
  /// The slice setting for text surface.
  Slice(SliceId),
  /// The host setting for text surface.
  Host,
}
