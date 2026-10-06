//! Storage and mutation of the module's buffered data.

use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use super::TextInputMode;

/// Grapheme-aligned text, cursor, and selection state with bounded normalized editing.
pub(super) struct TextBuffer {
  text: String,
  cursor: usize,
  anchor: Option<usize>,
  max_graphemes: Option<usize>,
  mode: TextInputMode,

  preferred_column: Option<usize>,
}

impl TextBuffer {
  /// Create a text buffer initialized from `text`, `max_graphemes`, `mode`.
  ///
  /// # Arguments
  ///
  /// * `text` - The text to process or display.
  /// * `max_graphemes` - The max graphemes.
  /// * `mode` - The mode.
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

  /// Return the text for the addressed object.
  pub fn text(&self) -> &str {
    &self.text
  }

  /// Update the text used by this text buffer.
  pub fn set_text(&mut self, text: String) -> bool {
    let text = truncate_graphemes(normalize(text, self.mode), self.max_graphemes);
    let changed = self.text != text;
    self.text = text;
    self.cursor = self.text.len();
    self.anchor = None;
    self.preferred_column = None;
    changed
  }

  /// Return the cursor for the addressed object.
  pub fn cursor(&self) -> usize {
    self.cursor
  }

  /// Return the selection for the addressed object when it is available.
  pub fn selection(&self) -> Option<Range<usize>> {
    let anchor = self.anchor?;
    (anchor != self.cursor).then_some(anchor.min(self.cursor)..anchor.max(self.cursor))
  }

  /// Return the text inside the current grapheme-aligned selection.
  pub fn selected_text(&self) -> Option<&str> {
    self.selection().map(|range| &self.text[range])
  }

  /// Select the entire text and report whether the selection changed.
  pub fn select_all(&mut self) -> bool {
    if self.text.is_empty() || self.selection() == Some(0..self.text.len()) {
      return false;
    }
    self.anchor = Some(0);
    self.cursor = self.text.len();
    self.preferred_column = None;
    true
  }

  /// Move the cursor to the nearest valid grapheme boundary and optionally extend the selection.
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

  /// Insert an allowed character at the cursor, replacing any active selection.
  pub fn insert_char(&mut self, ch: char) -> bool {
    (!ch.is_control())
      .then(|| ch.to_string())
      .is_some_and(|text| self.insert(&text))
  }

  /// Normalize and insert text at the cursor, replacing any active selection.
  pub fn insert_text(&mut self, text: &str) -> bool {
    let text = normalize(text.to_string(), self.mode);
    if text.is_empty() {
      return false;
    }
    self.insert(&text)
  }

  /// Insert a newline only when the input supports multiple lines.
  pub fn insert_newline(&mut self) -> bool {
    self.mode == TextInputMode::MultiLine && self.insert("\n")
  }

  /// Delete the selection or the grapheme immediately before the cursor.
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

  /// Delete the selection or the grapheme immediately after the cursor.
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

  /// Delete selected text and report whether a selection was removed.
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

  /// Move the cursor left by a grapheme or word and optionally extend the selection.
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

  /// Move the cursor right by a grapheme or word and optionally extend the selection.
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

  /// Move to a grapheme-aligned position and optionally extend the selection.
  pub fn move_to(&mut self, cursor: usize, extend: bool) -> bool {
    let changed = self.set_cursor(cursor, extend);
    self.preferred_column = None;
    changed
  }

  /// Return the current preferred column.
  pub fn preferred_column(&self) -> Option<usize> {
    self.preferred_column
  }

  /// Update the preferred column used by this text buffer.
  pub fn set_preferred_column(&mut self, column: Option<usize>) {
    self.preferred_column = column;
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
