/// Terminal capability description (Unicode, true color and mouse support).
#[derive(Clone, Debug)]
pub struct TerminalCapabilities {
  pub unicode: bool,
  pub truecolor: bool,
  pub mouse: bool,
}

impl TerminalCapabilities {
  /// Returns the default capabilities: Unicode enabled, true color and mouse disabled. The
  /// terminal itself is not probed.
  pub fn detect() -> Self {
    Self {
      unicode: true,
      truecolor: false,
      mouse: false,
    }
  }
}
