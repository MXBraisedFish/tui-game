use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::types::GraphemeInfo;
use crate::host_engine::services::rich_text::RichText;

/// 获取单个字符的终端列宽（0 表示组合字符或控制字符）。
pub fn char_width(ch: char) -> usize {
  UnicodeWidthChar::width(ch).unwrap_or(0)
}

/// 获取字符串在终端中的显示宽度（按 Unicode 列宽计算）。
pub fn display_width(text: &str) -> usize {
  UnicodeWidthStr::width(text)
}

/// 计算富文本的总显示宽度（所有分段的宽度之和）。
pub fn rich_text_width(rich_text: &RichText) -> usize {
  rich_text
    .segments
    .iter()
    .map(|segment| display_width(&segment.text))
    .sum()
}

/// 将字符串按字素边界拆分为 GraphemeInfo 列表。
pub fn graphemes(text: &str) -> Vec<GraphemeInfo> {
  UnicodeSegmentation::graphemes(text, true)
    .map(|g| GraphemeInfo {
      display_width: UnicodeWidthStr::width(g),
      text: g.to_string(),
    })
    .collect()
}

/// 获取单行字符串的总显示宽度（按字素宽度求和）。
pub fn line_display_width(line: &str) -> usize {
  graphemes(line).iter().map(|g| g.display_width).sum()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::rich_text::RichTextSegment;
  #[test]
  fn zero_width_zwj() {
    assert_eq!(display_width("\u{200D}"), 0);
    assert_eq!(char_width('\u{200D}'), 0);
  }

  #[test]
  fn zero_width_zws() {
    assert_eq!(display_width("\u{200B}"), 0);
    assert_eq!(char_width('\u{200B}'), 0);
  }

  #[test]
  fn zero_width_combining_mark() {

    assert_eq!(display_width("\u{0301}"), 0);
  }

  #[test]
  fn zero_width_variation_selector() {
    assert_eq!(display_width("\u{FE0F}"), 0);
    assert_eq!(display_width("\u{FE0E}"), 0);
  }

  #[test]
  fn wide_cjk() {
    assert_eq!(display_width("你好"), 4);
    assert_eq!(char_width('你'), 2);
    assert_eq!(char_width('好'), 2);
  }

  #[test]
  fn wide_emoji() {
    assert_eq!(display_width("😀"), 2);
    assert_eq!(display_width("🚀"), 2);
  }

  #[test]
  fn wide_fullwidth() {
    assert_eq!(display_width("Ａ"), 2);
    assert_eq!(char_width('Ａ'), 2);
  }

  #[test]
  fn normal_ascii() {
    assert_eq!(display_width("Hello"), 5);
    assert_eq!(char_width('a'), 1);
  }

  #[test]
  fn mixed_cjk_ascii() {
    assert_eq!(display_width("Hello世界"), 9);
  }

  #[test]
  fn mixed_with_zwj() {

    assert_eq!(display_width("a\u{200D}b"), 2);
  }

  #[test]
  fn grapheme_combining_accent() {

    let gs = graphemes("e\u{0301}");
    assert_eq!(gs.len(), 1);
    assert_eq!(gs[0].text, "e\u{0301}");
    assert_eq!(gs[0].display_width, 1);
  }

  #[test]
  fn grapheme_separate_chars() {
    let gs = graphemes("abc");
    assert_eq!(gs.len(), 3);
    assert_eq!(gs[0].display_width, 1);
    assert_eq!(gs[1].display_width, 1);
    assert_eq!(gs[2].display_width, 1);
  }

  #[test]
  fn grapheme_cjk() {
    let gs = graphemes("你好");
    assert_eq!(gs.len(), 2);
    assert_eq!(gs[0].display_width, 2);
    assert_eq!(gs[1].display_width, 2);
  }

  #[test]
  fn line_display_width_matches() {
    assert_eq!(line_display_width("Hello世界"), display_width("Hello世界"));
  }

  #[test]
  fn rich_text_width_basic() {
    let rt = RichText {
      segments: vec![RichTextSegment {
        text: "Hello世界".to_string(),
        style: Default::default(),
      }],
    };
    assert_eq!(rich_text_width(&rt), 9);
  }
}
