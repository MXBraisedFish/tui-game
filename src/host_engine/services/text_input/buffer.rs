use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use super::TextInputMode;

/// 文本缓冲区：管理文本内容、光标位置、选区，所有操作以字素（grapheme）为边界。
pub(super) struct TextBuffer {
  text: String,
  cursor: usize,
  anchor: Option<usize>,
  max_graphemes: Option<usize>,
  mode: TextInputMode,

  preferred_column: Option<usize>,
}

impl TextBuffer {
  pub fn new(text: String, max_graphemes: Option<usize>, mode: TextInputMode) -> Self {
    let text = normalize(text, mode);
    let text = truncate_graphemes(text, max_graphemes);
    let cursor = text.len();
    Self {
      text,
      cursor,
      anchor: None,
      max_graphemes,
      mode,
      preferred_column: None,
    }
  }

  pub fn text(&self) -> &str {
    &self.text
  }

  /// 替换全部文本，自动规范化和截断，返回是否实际变化。
  pub fn set_text(&mut self, text: String) -> bool {
    let text = truncate_graphemes(normalize(text, self.mode), self.max_graphemes);
    let changed = self.text != text;
    self.text = text;
    self.cursor = self.text.len();
    self.anchor = None;
    self.preferred_column = None;
    changed
  }

  pub fn clear(&mut self) -> bool {
    self.set_text(String::new())
  }

  pub fn cursor(&self) -> usize {
    self.cursor
  }

  pub fn selection(&self) -> Option<Range<usize>> {
    let anchor = self.anchor?;
    (anchor != self.cursor).then_some(anchor.min(self.cursor)..anchor.max(self.cursor))
  }

  pub fn selected_text(&self) -> Option<&str> {
    self.selection().map(|range| &self.text[range])
  }

  /// 全选文本，返回是否实际改变了选区。
  pub fn select_all(&mut self) -> bool {
    if self.text.is_empty() || self.selection() == Some(0..self.text.len()) {
      return false;
    }
    self.anchor = Some(0);
    self.cursor = self.text.len();
    self.preferred_column = None;
    true
  }

  /// 设置光标到最近的合法边界，可选扩展选区。
  pub fn set_cursor(&mut self, cursor: usize, extend: bool) -> bool {
    let cursor = self.closest_boundary(cursor);
    if cursor == self.cursor {
      if !extend && self.anchor.take().is_some() {
        return true;
      }
      return false;
    }
    if extend {
      self.anchor.get_or_insert(self.cursor);
    } else {
      self.anchor = None;
    }
    self.cursor = cursor;
    if self.anchor == Some(self.cursor) {
      self.anchor = None;
    }
    true
  }

  /// 插入单个字符（控制字符被忽略）。
  pub fn insert_char(&mut self, ch: char) -> bool {
    (!ch.is_control())
      .then(|| ch.to_string())
      .is_some_and(|text| self.insert(&text))
  }

  /// 插入文本字符串，自动规范化处理。
  pub fn insert_text(&mut self, text: &str) -> bool {
    let text = normalize(text.to_string(), self.mode);
    if text.is_empty() {
      return false;
    }
    self.insert(&text)
  }

  /// 插入换行符（仅多行模式有效）。
  pub fn insert_newline(&mut self) -> bool {
    self.mode == TextInputMode::MultiLine && self.insert("\n")
  }

  /// 删除光标前一个字素（有选区时先删除选区）。
  pub fn delete_prev(&mut self) -> bool {
    if self.delete_selection() {
      return true;
    }
    let Some(previous) = self
      .boundaries()
      .into_iter()
      .rev()
      .find(|end| *end < self.cursor)
    else {
      return false;
    };
    self.text.drain(previous..self.cursor);
    self.cursor = previous;
    self.preferred_column = None;
    true
  }

  /// 删除光标后一个字素（有选区时先删除选区）。
  pub fn delete_next(&mut self) -> bool {
    if self.delete_selection() {
      return true;
    }
    let Some(next) = self.boundaries().into_iter().find(|end| *end > self.cursor) else {
      return false;
    };
    self.text.drain(self.cursor..next);
    self.preferred_column = None;
    true
  }

  /// 删除当前选区内容，返回是否执行了删除。
  pub fn delete_selection(&mut self) -> bool {
    let Some(range) = self.selection() else {
      return false;
    };
    self.cursor = range.start;
    self.text.drain(range);
    self.anchor = None;
    self.preferred_column = None;
    true
  }

  pub fn move_left(&mut self) -> bool {
    self.move_left_select(false, false)
  }

  pub fn move_right(&mut self) -> bool {
    self.move_right_select(false, false)
  }

  /// 向左移动一个字素或一个单词，可选扩展选区。
  pub fn move_left_select(&mut self, extend: bool, word: bool) -> bool {
    if !extend && self.selection().is_some() {
      let start = self.selection().unwrap().start;
      return self.move_to(start, false);
    }
    let target = if word {
      self.word_start_left()
    } else {
      self
        .boundaries()
        .into_iter()
        .rev()
        .find(|end| *end < self.cursor)
    };
    target.is_some_and(|target| self.move_to(target, extend))
  }

  /// 向右移动一个字素或一个单词，可选扩展选区。
  pub fn move_right_select(&mut self, extend: bool, word: bool) -> bool {
    if !extend && self.selection().is_some() {
      let end = self.selection().unwrap().end;
      return self.move_to(end, false);
    }
    let target = if word {
      self.word_start_right()
    } else {
      self.boundaries().into_iter().find(|end| *end > self.cursor)
    };
    target.is_some_and(|target| self.move_to(target, extend))
  }

  pub fn move_home(&mut self) -> bool {
    self.move_to(0, false)
  }

  pub fn move_end(&mut self) -> bool {
    self.move_to(self.text.len(), false)
  }

  /// 移动光标到指定位置（自动对齐边界），可选扩展选区。
  pub fn move_to(&mut self, cursor: usize, extend: bool) -> bool {
    let changed = self.set_cursor(cursor, extend);
    self.preferred_column = None;
    changed
  }

  pub fn preferred_column(&self) -> Option<usize> {
    self.preferred_column
  }

  pub fn set_preferred_column(&mut self, column: Option<usize>) {
    self.preferred_column = column;
  }

  pub fn grapheme_count(&self) -> usize {
    self.text.graphemes(true).count()
  }

  fn boundaries(&self) -> Vec<usize> {
    std::iter::once(0)
      .chain(
        self
          .text
          .grapheme_indices(true)
          .map(|(start, grapheme)| start + grapheme.len()),
      )
      .collect()
  }

  fn closest_boundary(&self, cursor: usize) -> usize {
    self
      .boundaries()
      .into_iter()
      .take_while(|boundary| *boundary <= cursor)
      .last()
      .unwrap_or(0)
  }

  fn word_start_left(&self) -> Option<usize> {
    self
      .text
      .unicode_word_indices()
      .map(|(start, _)| start)
      .take_while(|start| *start < self.cursor)
      .last()
  }

  fn word_start_right(&self) -> Option<usize> {
    self
      .text
      .unicode_word_indices()
      .map(|(start, _)| start)
      .find(|start| *start > self.cursor)
      .or_else(|| (self.cursor < self.text.len()).then_some(self.text.len()))
  }

  fn insert(&mut self, value: &str) -> bool {
    let range = self.selection().unwrap_or(self.cursor..self.cursor);
    let mut accepted = String::new();
    for grapheme in value.graphemes(true) {
      let mut candidate = self.text.clone();
      candidate.replace_range(range.clone(), &(accepted.clone() + grapheme));
      if self
        .max_graphemes
        .is_some_and(|max| candidate.graphemes(true).count() > max)
      {
        break;
      }
      accepted.push_str(grapheme);
    }
    if accepted.is_empty() {
      return false;
    }
    self.text.replace_range(range.clone(), &accepted);
    self.cursor = range.start + accepted.len();
    self.anchor = None;
    self.preferred_column = None;
    true
  }
}

fn normalize(text: String, mode: TextInputMode) -> String {
  let text = text.replace("\r\n", "\n").replace('\r', "\n");
  match mode {
    TextInputMode::SingleLine => text.replace('\n', ""),
    TextInputMode::MultiLine => text,
  }
}

fn truncate_graphemes(mut text: String, max: Option<usize>) -> String {
  if let Some((end, _)) = max.and_then(|max| text.grapheme_indices(true).nth(max)) {
    text.truncate(end);
  }
  text
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn edits_and_moves_by_grapheme() {
    let mut buffer = TextBuffer::new("e\u{301}🌍我".into(), None, TextInputMode::SingleLine);
    assert!(buffer.move_left());
    assert!(buffer.delete_prev());
    assert_eq!(buffer.text(), "e\u{301}我");
    assert!(buffer.move_home());
    assert!(buffer.delete_next());
    assert_eq!(buffer.text(), "我");
  }

  #[test]
  fn selection_is_replaced_and_deleted() {
    let mut buffer = TextBuffer::new("a我👨‍👩".into(), None, TextInputMode::SingleLine);
    buffer.move_left_select(true, false);
    buffer.move_left_select(true, false);
    assert_eq!(buffer.selected_text(), Some("我👨‍👩"));
    assert!(buffer.insert_text("x"));
    assert_eq!(buffer.text(), "ax");
  }

  #[test]
  fn word_navigation_uses_unicode_words() {
    let mut buffer = TextBuffer::new("hello, 世界 next".into(), None, TextInputMode::SingleLine);
    assert!(buffer.move_left_select(false, true));
    assert_eq!(&buffer.text()[buffer.cursor()..], "next");
    assert!(buffer.move_left_select(false, true));
    assert_eq!(&buffer.text()[buffer.cursor()..], "界 next");
    assert!(buffer.move_left_select(false, true));
    assert_eq!(&buffer.text()[buffer.cursor()..], "世界 next");
    assert!(buffer.move_right_select(false, true));
    assert_eq!(&buffer.text()[buffer.cursor()..], "界 next");
  }

  #[test]
  fn paste_uses_fitting_grapheme_prefix() {
    let mut buffer = TextBuffer::new("a".into(), Some(3), TextInputMode::MultiLine);
    assert!(buffer.insert_text("我\n🌍"));
    assert_eq!(buffer.text(), "a我\n");
    assert_eq!(buffer.grapheme_count(), 3);
  }

  #[test]
  fn set_text_normalizes_and_clears_selection() {
    let mut buffer = TextBuffer::new("ab".into(), Some(2), TextInputMode::SingleLine);
    buffer.move_left_select(true, false);
    assert!(buffer.set_text("a\r\n我🌍".into()));
    assert_eq!(buffer.text(), "a我");
    assert_eq!(buffer.cursor(), buffer.text().len());
    assert_eq!(buffer.selection(), None);

    assert!(buffer.move_left_select(true, false));
    assert!(!buffer.set_text("a我".into()));
    assert_eq!(buffer.cursor(), buffer.text().len());
    assert_eq!(buffer.selection(), None);
  }
}
