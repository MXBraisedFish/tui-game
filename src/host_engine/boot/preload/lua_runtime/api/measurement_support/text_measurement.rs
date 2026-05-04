//! 文本尺寸计算

use unicode_width::UnicodeWidthStr;

/// 文本尺寸。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextSize {
    pub width: u16,
    pub height: u16,
}

/// 计算文本占用的终端字符宽高。
pub fn measure_text(text: &str) -> TextSize {
    if text.is_empty() {
        return TextSize {
            width: 0,
            height: 0,
        };
    }

    let mut width = 0usize;
    let mut height = 0usize;

    for line in text.split('\n') {
        width = width.max(UnicodeWidthStr::width(line));
        height += 1;
    }

    TextSize {
        width: width.min(usize::from(u16::MAX)) as u16,
        height: height.min(usize::from(u16::MAX)) as u16,
    }
}
