//! Visible text measurements that account for terminal column widths.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::types::GraphemeInfo;

/// Return the terminal column width of one Unicode scalar value.
pub fn char_width(ch: char) -> usize {
  UnicodeWidthChar::width(ch).unwrap_or(0)
}

/// Return the terminal column width of the supplied text.
pub fn display_width(text: &str) -> usize {
  UnicodeWidthStr::width(text)
}

/// Split text into graphemes and retain each grapheme's text and terminal column width.
pub fn graphemes(text: &str) -> Vec<GraphemeInfo> {
  UnicodeSegmentation::graphemes(text, true)
    .map(|g| GraphemeInfo {
      display_width: UnicodeWidthStr::width(g),
      text: g.to_string(),
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

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
}
