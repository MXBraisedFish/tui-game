use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::types::GraphemeInfo;
use crate::host_engine::services::rich_text::RichText;

// ── 基础宽度计算 ──

/// 单字符终端显示宽度。
///
/// 零宽字符（ZWJ、ZWS、组合标记等）返回 0，
/// CJK / 全角 / emoji 返回 2，
/// 普通 ASCII / 拉丁字符返回 1。
pub fn char_width(ch: char) -> usize {
  UnicodeWidthChar::width(ch).unwrap_or(0)
}

/// 文本的终端显示宽度（使用 Unicode 标准东亚宽度规则）。
///
/// 等价于对每个 grapheme cluster 宽度求和。
pub fn display_width(text: &str) -> usize {
  UnicodeWidthStr::width(text)
}

/// 富文本的终端显示宽度（去标签后计算）。
pub fn rich_text_width(rich_text: &RichText) -> usize {
  rich_text
    .segments
    .iter()
    .map(|segment| display_width(&segment.text))
    .sum()
}

// ── Grapheme 切分 ──

/// 将文本切分为 grapheme cluster 列表，每个附带其终端显示宽度。
///
/// Grapheme cluster 是用户感知的"一个字符"——
/// 例如 `e\u{0301}`（é）是 2 个 Rust `char`，但只有 1 个 grapheme。
/// Canvas 写入时必须以 grapheme 为单位推进光标，
/// 否则组合标记会被错误地放入独立单元格。
pub fn graphemes(text: &str) -> Vec<GraphemeInfo> {
  UnicodeSegmentation::graphemes(text, true)
    .map(|g| GraphemeInfo {
      display_width: UnicodeWidthStr::width(g),
      text: g.to_string(),
    })
    .collect()
}

/// 单行文本的终端显示宽度（grapheme 求和版本）。
///
/// 结果应与 [`display_width`] 一致，但提供 grapheme 级别的中间结果
/// 供调试和逐 grapheme 处理流程使用。
pub fn line_display_width(line: &str) -> usize {
  graphemes(line).iter().map(|g| g.display_width).sum()
}

#[cfg(test)]
mod tests {
  use super::*;

  // ── 零宽字符 ──

  #[test]
  fn zero_width_zwj() {
    assert_eq!(display_width("\u{200D}"), 0); // ZWJ
    assert_eq!(char_width('\u{200D}'), 0);
  }

  #[test]
  fn zero_width_zws() {
    assert_eq!(display_width("\u{200B}"), 0); // ZWS
    assert_eq!(char_width('\u{200B}'), 0);
  }

  #[test]
  fn zero_width_combining_mark() {
    // 组合锐音符本身是零宽
    assert_eq!(display_width("\u{0301}"), 0);
  }

  #[test]
  fn zero_width_variation_selector() {
    assert_eq!(display_width("\u{FE0F}"), 0); // VS16 emoji style
    assert_eq!(display_width("\u{FE0E}"), 0); // VS15 text style
  }

  // ── 宽字符 ──

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
    assert_eq!(display_width("Ａ"), 2); // fullwidth A
    assert_eq!(char_width('Ａ'), 2);
  }

  // ── 普通字符 ──

  #[test]
  fn normal_ascii() {
    assert_eq!(display_width("Hello"), 5);
    assert_eq!(char_width('a'), 1);
  }

  // ── 混合文本 ──

  #[test]
  fn mixed_cjk_ascii() {
    assert_eq!(display_width("Hello世界"), 9); // 5 + 2 + 2
  }

  #[test]
  fn mixed_with_zwj() {
    // ZWJ 不占宽
    assert_eq!(display_width("a\u{200D}b"), 2);
  }

  // ── Grapheme cluster ──

  #[test]
  fn grapheme_combining_accent() {
    // e + combining acute = é（2 个 char，1 个 grapheme，宽 1）
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

  // ── rich_text_width ──

  #[test]
  fn rich_text_width_basic() {
    let rt = RichText {
      segments: vec![crate::host_engine::services::RichTextSegment {
        text: "Hello世界".to_string(),
        style: Default::default(),
      }],
    };
    assert_eq!(rich_text_width(&rt), 9);
  }
}
