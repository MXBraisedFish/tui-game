// 图片协议枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageProtocol {
  None,
  Kitty,
  Sixel,
  ITerm2,
}

/// 自动检测结果（供终端检测向导预选选项）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectionResult {
  /// 是否支持真彩（COLORTERM 含 truecolor / 24bit）
  pub truecolor: bool,
  /// 检测到的图片协议
  pub image_protocol: ImageProtocol,
}

impl Default for DetectionResult {
  fn default() -> Self {
    Self {
      truecolor: false,
      image_protocol: ImageProtocol::None,
    }
  }
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
