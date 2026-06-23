use unicode_segmentation::UnicodeSegmentation;

use super::TextInputMode;

pub(super) struct TextBuffer {
  text: String,
  cursor: usize,
  max_graphemes: Option<usize>,
  mode: TextInputMode,
}

impl TextBuffer {
  pub fn new(text: String, max_graphemes: Option<usize>, mode: TextInputMode) -> Self {
    let text = normalize(text, max_graphemes, mode);
    let cursor = text.len();
    Self {
      text,
      cursor,
      max_graphemes,
      mode,
    }
  }

  pub fn text(&self) -> &str {
    &self.text
  }

  pub fn set_text(&mut self, text: String) -> bool {
    let text = normalize(text, self.max_graphemes, self.mode);
    if self.text == text {
      return false;
    }
    self.text = text;
    self.cursor = self.text.len();
    true
  }

  pub fn clear(&mut self) -> bool {
    self.set_text(String::new())
  }

  pub fn cursor(&self) -> usize {
    self.cursor
  }

  pub fn insert_char(&mut self, ch: char) -> bool {
    if ch.is_control() {
      return false;
    }

    let mut next = self.text.clone();
    next.insert(self.cursor, ch);
    if self
      .max_graphemes
      .is_some_and(|max| next.graphemes(true).count() > max)
    {
      return false;
    }

    let inserted_end = self.cursor + ch.len_utf8();
    self.text = next;
    self.cursor = self
      .text
      .grapheme_indices(true)
      .map(|(start, grapheme)| start + grapheme.len())
      .find(|end| *end >= inserted_end)
      .unwrap_or(self.text.len());
    true
  }

  pub fn insert_newline(&mut self) -> bool {
    if self.mode != TextInputMode::MultiLine {
      return false;
    }
    self.insert("\n")
  }

  pub fn delete_prev(&mut self) -> bool {
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
    true
  }

  pub fn delete_next(&mut self) -> bool {
    let Some(next) = self.boundaries().into_iter().find(|end| *end > self.cursor) else {
      return false;
    };
    self.text.drain(self.cursor..next);
    true
  }

  pub fn move_left(&mut self) -> bool {
    let Some(previous) = self
      .boundaries()
      .into_iter()
      .rev()
      .find(|end| *end < self.cursor)
    else {
      return false;
    };
    self.cursor = previous;
    true
  }

  pub fn move_right(&mut self) -> bool {
    let Some(next) = self.boundaries().into_iter().find(|end| *end > self.cursor) else {
      return false;
    };
    self.cursor = next;
    true
  }

  pub fn move_home(&mut self) -> bool {
    if self.cursor == 0 {
      return false;
    }
    self.cursor = 0;
    true
  }

  pub fn move_end(&mut self) -> bool {
    if self.cursor == self.text.len() {
      return false;
    }
    self.cursor = self.text.len();
    true
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

  fn insert(&mut self, value: &str) -> bool {
    let mut next = self.text.clone();
    next.insert_str(self.cursor, value);
    if self
      .max_graphemes
      .is_some_and(|max| next.graphemes(true).count() > max)
    {
      return false;
    }
    self.text = next;
    self.cursor += value.len();
    true
  }
}

fn normalize(text: String, max_graphemes: Option<usize>, mode: TextInputMode) -> String {
  let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
  let mut text = match mode {
    TextInputMode::SingleLine => normalized.replace('\n', ""),
    TextInputMode::MultiLine => normalized,
  };
  if let Some(max) = max_graphemes {
    if let Some((end, _)) = text.grapheme_indices(true).nth(max) {
      text.truncate(end);
    }
  }
  text
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn insert_ascii_and_cjk() {
    let mut buffer = TextBuffer::new(String::new(), None, TextInputMode::SingleLine);
    assert!(buffer.insert_char('a'));
    assert!(buffer.insert_char('我'));
    assert_eq!(buffer.text(), "a我");
    assert_eq!(buffer.grapheme_count(), 2);
  }

  #[test]
  fn backspace_and_delete_remove_graphemes() {
    let mut buffer = TextBuffer::new("a👨‍👩我".to_string(), None, TextInputMode::SingleLine);
    assert!(buffer.move_left());
    assert!(buffer.delete_prev());
    assert_eq!(buffer.text(), "a我");
    assert!(buffer.move_home());
    assert!(buffer.delete_next());
    assert_eq!(buffer.text(), "我");
  }

  #[test]
  fn cursor_moves_by_grapheme() {
    let mut buffer = TextBuffer::new("e\u{301}🌍我".to_string(), None, TextInputMode::SingleLine);
    let end = buffer.cursor();
    assert!(buffer.move_left());
    let after_cjk = buffer.cursor();
    assert!(buffer.move_left());
    let after_emoji = buffer.cursor();
    assert!(buffer.move_left());
    assert_eq!(buffer.cursor(), 0);
    assert!(after_emoji > 0 && after_cjk > after_emoji && end > after_cjk);
    assert!(buffer.move_end());
    assert_eq!(buffer.cursor(), end);
  }

  #[test]
  fn max_graphemes_blocks_insert() {
    let mut buffer = TextBuffer::new("a".to_string(), Some(1), TextInputMode::SingleLine);
    assert!(!buffer.insert_char('b'));
    assert!(buffer.insert_char('\u{301}'));
    assert_eq!(buffer.text(), "a\u{301}");
    assert_eq!(buffer.grapheme_count(), 1);
  }

  #[test]
  fn max_graphemes_counts_newline_as_one_character() {
    let mut buffer = TextBuffer::new("a\nb".to_string(), Some(2), TextInputMode::MultiLine);
    assert_eq!(buffer.text(), "a\n");
    assert_eq!(buffer.grapheme_count(), 2);
    assert!(!buffer.insert_char('b'));
    assert!(!buffer.insert_newline());
  }

  #[test]
  fn set_text_normalizes_and_moves_cursor_to_end() {
    let mut buffer = TextBuffer::new(String::new(), Some(2), TextInputMode::SingleLine);
    assert!(buffer.set_text("a\r\n我🌍".to_string()));
    assert_eq!(buffer.text(), "a我");
    assert_eq!(buffer.cursor(), buffer.text().len());
    assert!(buffer.clear());
    assert_eq!(buffer.text(), "");
    assert_eq!(buffer.cursor(), 0);
  }

  #[test]
  fn multiline_normalizes_and_inserts_newlines() {
    let mut buffer = TextBuffer::new("a\r\nb\rc".to_string(), None, TextInputMode::MultiLine);
    assert_eq!(buffer.text(), "a\nb\nc");
    assert!(buffer.insert_newline());
    assert_eq!(buffer.text(), "a\nb\nc\n");

    let mut single = TextBuffer::new("a\r\nb".to_string(), None, TextInputMode::SingleLine);
    assert_eq!(single.text(), "ab");
    assert!(!single.insert_newline());
  }
}
