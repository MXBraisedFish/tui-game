//! Text clipboard access with an unavailable-backend fallback.
//!
//! System availability and clipboard contents vary. Read text only when the returned value is
//! `Some`; `None` also covers unavailable backends and non-text clipboard contents.
//!
//! # Examples
//!
//! ```rust
//! use tg_service_clipboard::ClipboardService;
//!
//! let mut clipboard = ClipboardService::new();
//! if let Some(text) = clipboard.read_text() {
//!     let _character_count = text.chars().count();
//! }
//! ```

/// The public entry point for clipboard operations.
pub struct ClipboardService {
  clipboard: Option<arboard::Clipboard>,
  last_error: Option<String>,
}

impl std::fmt::Debug for ClipboardService {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter
      .debug_struct("ClipboardService")
      .field("available", &self.clipboard.is_some())
      .field("last_error", &self.last_error)
      .finish_non_exhaustive()
  }
}

impl ClipboardService {
  /// Open the system clipboard, retaining an unavailable backend if initialization fails.
  pub fn new() -> Self {
    let clipboard = arboard::Clipboard::new().ok();

    let last_error = clipboard
      .is_none()
      .then(|| "Failed to open system clipboard".to_string());
    Self {
      clipboard,
      last_error,
    }
  }

  /// Return system clipboard text, or `None` when the backend or text contents are unavailable.
  pub fn read_text(&mut self) -> Option<String> {
    let clipboard = self.clipboard.as_mut()?;

    match clipboard.get_text() {
      Ok(text) => Some(text),
      Err(_) => {
        self.last_error = Some("Failed to read from clipboard".to_string());
        None
      }
    }
  }

  /// Replace system clipboard text and return whether the backend accepted the write.
  pub fn write_text(&mut self, text: &str) -> bool {
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

  /// Create a clipboard service without an active platform backend.
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
