//! 文本尺寸计算

use unicode_width::UnicodeWidthStr;

use crate::host_engine::boot::preload::lua_runtime::api::text_support::text_wrapping;

/// 文本尺寸。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextSize {
    pub width: u16,
    pub height: u16,
}

/// 计算文本占用的终端字符宽高。
pub fn measure_text(text: &str, wrap_width: Option<u16>) -> TextSize {
    if text.is_empty() {
        return TextSize {
            width: 0,
            height: 0,
        };
    }

    let lines = text_wrapping::wrap_text_lines(text, wrap_width);
    let width = if wrap_width.is_some() {
        text_wrapping::max_line_width(&lines)
    } else {
        lines
            .iter()
            .map(|line| UnicodeWidthStr::width(line.as_str()))
            .max()
            .unwrap_or_default()
    };
    let height = lines.len();

    TextSize {
        width: width.min(usize::from(u16::MAX)) as u16,
        height: height.min(usize::from(u16::MAX)) as u16,
    }
}
