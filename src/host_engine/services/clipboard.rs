/// 剪贴板服务，提供系统剪贴板的读写能力
pub struct ClipboardService {
  clipboard: Option<arboard::Clipboard>,
  last_error: Option<String>,
  #[cfg(test)]
  memory: Option<String>,
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
      #[cfg(test)]
      memory: None,
    }
  }

  /// 读取剪贴板中的文本内容
  pub fn read_text(&mut self) -> Option<String> {
    #[cfg(test)]
    if let Some(text) = &self.memory {
      return Some(text.clone());
    }
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

  /// 向剪贴板写入文本
  pub fn write_text(&mut self, text: &str) -> bool {
    #[cfg(test)]
    if self.memory.is_some() {
      self.memory = Some(text.to_string());
      return true;
    }
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
  pub(crate) fn memory(text: &str) -> Self {
    Self {
      clipboard: None,
      last_error: None,
      memory: Some(text.to_string()),
    }
  }

  #[cfg(test)]
  pub(crate) fn unavailable() -> Self {
    Self {
      clipboard: None,
      last_error: None,
      memory: None,
    }
  }
}
