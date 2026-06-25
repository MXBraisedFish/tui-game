/// 终端能力描述（Unicode / 真彩色 / 鼠标支持）
#[derive(Clone, Debug)]
pub struct TerminalCapabilities {
  pub unicode: bool,
  pub truecolor: bool,
  pub mouse: bool,
}

impl TerminalCapabilities {
  /// 检测当前终端的能力
  pub fn detect() -> Self {
    Self {
      unicode: true,
      truecolor: false,
      mouse: false,
    }
  }
}
