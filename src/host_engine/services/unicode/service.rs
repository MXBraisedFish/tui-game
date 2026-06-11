use super::measure;
use super::types::GraphemeInfo;
use crate::host_engine::services::rich_text::RichText;

pub struct UnicodeService;

impl UnicodeService {
  pub fn new() -> Self {
    Self
  }

  // ── 基础宽度 ──

  /// 单字符终端显示宽度。
  pub fn char_width(&self, ch: char) -> usize {
    measure::char_width(ch)
  }

  /// 文本的终端显示宽度。
  pub fn display_width(&self, text: &str) -> usize {
    measure::display_width(text)
  }

  /// 富文本的终端显示宽度（去标签后计算）。
  pub fn rich_text_width(&self, rich_text: &RichText) -> usize {
    measure::rich_text_width(rich_text)
  }

  // ── Grapheme 切分 ──

  /// 将文本切分为 grapheme cluster 列表，每个附带其终端显示宽度。
  pub fn graphemes(&self, text: &str) -> Vec<GraphemeInfo> {
    measure::graphemes(text)
  }

  /// 单行文本的终端显示宽度（grapheme 求和版本）。
  pub fn line_display_width(&self, line: &str) -> usize {
    measure::line_display_width(line)
  }
}
