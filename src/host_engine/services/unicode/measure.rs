use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::host_engine::services::rich_text::RichText;

// 普通文本显示宽度
pub fn display_width(text: &str) -> usize {
  UnicodeWidthStr::width(text)
}

// 单字符显示宽度
pub fn char_width(ch: char) -> usize {
  UnicodeWidthChar::width(ch).unwrap_or(0)
}

// 富文本显示宽度
pub fn rich_text_width(rich_text: &RichText) -> usize {
  rich_text
    .segments
    .iter()
    .map(|segment| display_width(&segment.text))
    .sum()
}
