//! 终端颜色能力检测

/// 终端颜色支持能力
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorSupport {
    Basic,
    Ansi256,
    TrueColor,
}

/// 检测终端颜色能力
pub fn detect() -> ColorSupport {
    let color_term = std::env::var("COLORTERM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if color_term.contains("truecolor") || color_term.contains("24bit") {
        return ColorSupport::TrueColor;
    }

    let term = std::env::var("TERM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if term.contains("256color") {
        return ColorSupport::Ansi256;
    }

    ColorSupport::Basic
}
