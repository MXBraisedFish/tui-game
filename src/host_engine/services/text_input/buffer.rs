use unicode_segmentation::UnicodeSegmentation;

pub(super) struct TextBuffer {
  text: String,
  cursor: usize,
  max_graphemes: Option<usize>,
}

impl TextBuffer {
  pub fn new(text: String, max_graphemes: Option<usize>) -> Self {
    let text = normalize(text, max_graphemes);
    let cursor = text.len();
    Self {
      text,
      cursor,
      max_graphemes,
    }
  }

  pub fn text(&self) -> &str {
    &self.text
  }

  pub fn set_text(&mut self, text: String) -> bool {
    let text = normalize(text, self.max_graphemes);
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
}

fn normalize(text: String, max_graphemes: Option<usize>) -> String {
  let mut text: String = text
    .chars()
    .filter(|ch| !matches!(ch, '\r' | '\n'))
    .collect();
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
    let mut buffer = TextBuffer::new(String::new(), None);
    assert!(buffer.insert_char('a'));
    assert!(buffer.insert_char('我'));
    assert_eq!(buffer.text(), "a我");
    assert_eq!(buffer.grapheme_count(), 2);
  }

  #[test]
  fn backspace_and_delete_remove_graphemes() {
    let mut buffer = TextBuffer::new("a👨‍👩我".to_string(), None);
    assert!(buffer.move_left());
    assert!(buffer.delete_prev());
    assert_eq!(buffer.text(), "a我");
    assert!(buffer.move_home());
    assert!(buffer.delete_next());
    assert_eq!(buffer.text(), "我");
  }

  #[test]
  fn cursor_moves_by_grapheme() {
    let mut buffer = TextBuffer::new("e\u{301}🌍我".to_string(), None);
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
    let mut buffer = TextBuffer::new("a".to_string(), Some(1));
    assert!(!buffer.insert_char('b'));
    assert!(buffer.insert_char('\u{301}'));
    assert_eq!(buffer.text(), "a\u{301}");
    assert_eq!(buffer.grapheme_count(), 1);
  }

  #[test]
  fn set_text_normalizes_and_moves_cursor_to_end() {
    let mut buffer = TextBuffer::new(String::new(), Some(2));
    assert!(buffer.set_text("a\r\n我🌍".to_string()));
    assert_eq!(buffer.text(), "a我");
    assert_eq!(buffer.cursor(), buffer.text().len());
    assert!(buffer.clear());
    assert_eq!(buffer.text(), "");
    assert_eq!(buffer.cursor(), 0);
  }
}
