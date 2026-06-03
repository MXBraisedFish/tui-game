use super::measure;
use crate::host_engine::services::rich_text::RichText;

pub struct UnicodeService;

impl UnicodeService {
  pub fn new() -> Self {
    Self
  }

  // 普通文本显示宽度
  pub fn display_width(&self, text: &str) -> usize {
    measure::display_width(text)
  }

  // 单字符显示宽度
  pub fn char_width(&self, ch: char) -> usize {
    measure::char_width(ch)
  }

  // 富文本显示宽度
  pub fn rich_text_width(&self, rich_text: &RichText) -> usize {
    measure::rich_text_width(rich_text)
  }
}
