//! Clipboard service: system clipboard text read/write.

/// Clipboard service that reads and writes the system clipboard.
pub struct ClipboardService {
  clipboard: Option<arboard::Clipboard>,
  last_error: Option<String>,
}

impl ClipboardService {
  pub fn new() -> Self {
    let clipboard = arboard::Clipboard::new().ok();
    // TODO: add log warn when LogService is available
    let last_error = clipboard
      .is_none()
      .then(|| "Failed to open system clipboard".to_string());
    Self {
      clipboard,
      last_error,
    }
  }

  /// Reads the text content of the clipboard; returns `None` when the clipboard is unavailable or
  /// cannot be read.
  pub fn read_text(&mut self) -> Option<String> {
    let clipboard = self.clipboard.as_mut()?;
    // TODO: add log warn when LogService is available
    match clipboard.get_text() {
      Ok(text) => Some(text),
      Err(_) => {
        self.last_error = Some("Failed to read from clipboard".to_string());
        None
      }
    }
  }

  /// Writes text to the clipboard and returns whether it succeeded.
  pub fn write_text(&mut self, text: &str) -> bool {
    // TODO: add log warn when LogService is available
    match self.clipboard.as_mut() {
      Some(clipboard) => match clipboard.set_text(text) {
        Ok(()) => true,
        Err(_) => {
          self.last_error = Some("Failed to write to clipboard".to_string());
          false
        }
      },
      None => {
        self.last_error = Some("Clipboard not available".to_string());
        false
      }
    }
  }

  #[cfg(test)]
  pub(crate) fn unavailable() -> Self {
    Self {
      clipboard: None,
      last_error: None,
    }
  }
}

impl Default for ClipboardService {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unavailable_clipboard_reads_nothing_and_rejects_writes() {
    let mut clipboard = ClipboardService::unavailable();
    assert_eq!(clipboard.read_text(), None);
    assert!(!clipboard.write_text("text"));
    assert_eq!(
      clipboard.last_error.as_deref(),
      Some("Clipboard not available")
    );
  }
}
