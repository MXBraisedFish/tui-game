//! 文本尺寸计算

use unicode_width::UnicodeWidthStr;

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::{
    WrapLimit, WrapOptions,
};
use crate::host_engine::boot::preload::lua_runtime::api::text_support::text_wrapping;

/// 文本尺寸。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextSize {
    pub width: u16,
    pub height: u16,
}

/// 计算文本占用的终端字符宽高。
pub fn measure_text(text: &str, wrap_width: Option<u16>) -> TextSize {
    measure_text_with_options(
        text,
        &WrapOptions {
            wrap_width: wrap_width.map_or(WrapLimit::Disabled, WrapLimit::Fixed),
            ..WrapOptions::default()
        },
    )
}

/// 按换行配置计算文本占用的终端字符宽高。
pub fn measure_text_with_options(text: &str, wrap_options: &WrapOptions) -> TextSize {
    if text.is_empty() {
        return TextSize {
            width: 0,
            height: 0,
        };
    }

    let lines = text_wrapping::wrap_text_lines_with_options(text, wrap_options);
    let width = if wrap_options.wrap_width != WrapLimit::Disabled {
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
