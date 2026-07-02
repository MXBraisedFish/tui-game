use crate::host_engine::services::{TerminalColor, TextColor, TextStyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProgressBarId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressBarFillOrigin {
  Left,
  Right,
  Center,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressBarSegmentStyle {
  pub ch: char,
  pub style: TextStyle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressBarOptions {
  pub completed: ProgressBarSegmentStyle,
  pub preview: ProgressBarSegmentStyle,
  pub remaining: ProgressBarSegmentStyle,
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
