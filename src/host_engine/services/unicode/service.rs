use super::measure;
use super::types::GraphemeInfo;

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

  pub fn graphemes(&self, text: &str) -> Vec<GraphemeInfo> {
    measure::graphemes(text)
  }
}
