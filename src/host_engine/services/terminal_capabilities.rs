// 图片协议枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ImageProtocol {
  None,
  Kitty,
  Sixel,
  ITerm2,
}

// 终端能力结构体
#[derive(Clone, Debug)]
pub struct TerminalCapabilities {
  pub unicode: bool,
  pub truecolor: bool,
  pub mouse: bool,
  pub image_protocol: ImageProtocol,
}

impl TerminalCapabilities {
  pub fn detect() -> Self {
    Self {
      unicode: true,
      truecolor: false,
      mouse: false,
      image_protocol: ImageProtocol::None,
    }
  }
}
