//! Identifiers, configuration, states, and events shared by this module.

use tg_core_style::{TerminalColor, TextColor, TextStyle};

/// The identity of progress bar within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProgressBarId(
  /// The wrapped u64 value.
  pub u64,
);

/// The edge from which the progress bar fills.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressBarFillOrigin {
  /// The left setting for progress bar fill origin.
  Left,
  /// The right setting for progress bar fill origin.
  Right,
  /// The center setting for progress bar fill origin.
  Center,
}

/// The progress bar segment style representation used by this module.
///
/// # Fields
///
/// * `ch` - The ch.
/// * `style` - The text style applied to the rendered content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressBarSegmentStyle {
  /// The ch.
  pub ch: char,
  /// The text style applied to the rendered content.
  pub style: TextStyle,
}

/// Configuration values controlling progress bar behavior.
///
/// # Fields
///
/// * `completed` - The completed.
/// * `preview` - The preview.
/// * `remaining` - The remaining.
/// * `origin` - The origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressBarOptions {
  /// The completed.
  pub completed: ProgressBarSegmentStyle,
  /// The preview.
  pub preview: ProgressBarSegmentStyle,
  /// The remaining.
  pub remaining: ProgressBarSegmentStyle,
  /// The origin.
  pub origin: ProgressBarFillOrigin,
}

impl Default for ProgressBarSegmentStyle {
  fn default() -> Self {
    Self {
      ch: '█',
      style: TextStyle {
        foreground: Some(TextColor::Terminal(TerminalColor::White)),
        background: Some(TextColor::Transparent),
        ..Default::default()
      },
    }
  }
}

impl Default for ProgressBarOptions {
  fn default() -> Self {
    Self {
      completed: ProgressBarSegmentStyle {
        ch: '█',
        style: TextStyle {
          foreground: Some(TextColor::Terminal(TerminalColor::Green)),
          background: Some(TextColor::Transparent),
          ..Default::default()
        },
      },
      preview: ProgressBarSegmentStyle {
        ch: '█',
        style: TextStyle {
          foreground: Some(TextColor::Terminal(TerminalColor::BrightBlue)),
          background: Some(TextColor::Transparent),
          ..Default::default()
        },
      },
      remaining: ProgressBarSegmentStyle {
        ch: '─',
        style: TextStyle {
          foreground: Some(TextColor::Terminal(TerminalColor::White)),
          background: Some(TextColor::Transparent),
          ..Default::default()
        },
      },
      origin: ProgressBarFillOrigin::Left,
    }
  }
}
