// TODO：目前的能力检测100%不可信，因此需要依靠用户选择，使用自动检测+引导来，包含真彩、图片协议、鼠标支持（后续加上鼠标事件）

// 图片协议枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageProtocol {
  None,
  Kitty,
  Sixel,
  ITerm2
}

// 终端能力结构体
#[derive(Clone, Debug)]
pub struct TerminalCapabilities {
  pub unicode: bool,
  pub truecolor: bool,
  pub mouse: bool,
  pub image_protocol: ImageProtocol
}

impl TerminalCapabilities {
  pub fn detect() -> Self {
    Self {
      unicode: detect_unicode(),
      truecolor: detect_truecolor(),
      mouse: false,
      image_protocol: detect_image_protocol()
    }
  }
}

// 是否支持Unicode
fn detect_unicode() -> bool {
  true
}

// 是否支持真彩
fn detect_truecolor() -> bool {
  // 读取环境变量的颜色支持
  std::env::var("COLOTERM").map(|v| {
    // 闭包内的小写转换
    let v = v.to_lowercase();
    // 是否包含真彩或者24位色彩（其实俩一样）
    v.contains("truecolor") || v.contains("24bit")
  }).unwrap_or(false)
}

// 是否支持图片协议
fn detect_image_protocol() -> ImageProtocol {
  // Kitty 协议检测
  // 直接检查对应的环境变量即可
  if std::env::var("KITTY_WINDOW_ID").is_ok() {
    return ImageProtocol::Kitty;
  }

  // Sixel 协议检测
  // 读TERM变量，然后读取sixel字段
  if std::env::var("TERM")
    .map(|v| v.contains("sixel"))
    .unwrap_or(false)
  {
    return ImageProtocol::Sixel;
  }

  // iTerm2 协议检测
  // 读环境变量，然后字段完整检测
  if std::env::var("TERM_PROGRAM")
    .map(|v| v == "iTerm.app")
    .unwrap_or(false)
  {
    return ImageProtocol::ITerm2;
  }

  // 不支持任何图片协议
  ImageProtocol::None
}