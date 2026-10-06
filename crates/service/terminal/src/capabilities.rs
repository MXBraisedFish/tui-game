//! Detection and overrides for terminal color and display capabilities.

/// Detected or overridden Unicode, mouse, and color support.
///
/// # Fields
///
/// * `unicode` - Whether Unicode output is supported.
/// * `truecolor` - Whether full RGB output is supported.
/// * `mouse` - Whether terminal pointer events are supported.
#[derive(Clone, Debug)]
pub struct TerminalCapabilities {
  /// Whether Unicode output is supported.
  pub unicode: bool,
  /// Whether full RGB output is supported.
  pub truecolor: bool,
  /// Whether terminal pointer events are supported.
  pub mouse: bool,
}

impl TerminalCapabilities {
  /// Create conservative initial capabilities: Unicode enabled, RGB and pointer support unconfirmed.
  pub fn detect() -> Self {
    Self {
      unicode: true,
      truecolor: false,
      mouse: false,
    }
  }
}
