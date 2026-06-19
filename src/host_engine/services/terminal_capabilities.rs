// 终端能力结构体
#[derive(Clone, Debug)]
pub struct TerminalCapabilities {
  pub unicode: bool,
  pub truecolor: bool,
  pub mouse: bool,
}

impl TerminalCapabilities {
  pub fn detect() -> Self {
    Self {
      unicode: true,
      truecolor: false,
      mouse: false,
    }
  }
}
