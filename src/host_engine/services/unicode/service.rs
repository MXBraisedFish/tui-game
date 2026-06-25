use super::measure;
use super::types::GraphemeInfo;
use crate::host_engine::services::rich_text::RichText;

/// Unicode 服务：封装字符宽度测量、字素拆分等 Unicode 相关工具方法。
pub struct UnicodeService;

impl UnicodeService {
  pub fn new() -> Self {
    Self
  }

  pub fn char_width(&self, ch: char) -> usize {
    measure::char_width(ch)
  }

  pub fn display_width(&self, text: &str) -> usize {
    measure::display_width(text)
  }

  pub fn rich_text_width(&self, rich_text: &RichText) -> usize {
    measure::rich_text_width(rich_text)
  }

  pub fn graphemes(&self, text: &str) -> Vec<GraphemeInfo> {
    measure::graphemes(text)
  }

  pub fn line_display_width(&self, line: &str) -> usize {
    measure::line_display_width(line)
  }
}
