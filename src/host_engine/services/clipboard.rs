pub struct ClipboardService {
  clipboard: Option<arboard::Clipboard>,
  #[cfg(test)]
  memory: Option<String>,
}

impl ClipboardService {
  pub fn new() -> Self {
    Self {
      clipboard: arboard::Clipboard::new().ok(),
      #[cfg(test)]
      memory: None,
    }
  }

  pub fn read_text(&mut self) -> Option<String> {
    #[cfg(test)]
    if let Some(text) = &self.memory {
      return Some(text.clone());
    }
    self.clipboard.as_mut()?.get_text().ok()
  }

  pub fn write_text(&mut self, text: &str) -> bool {
    #[cfg(test)]
    if self.memory.is_some() {
      self.memory = Some(text.to_string());
      return true;
    }
    self
      .clipboard
      .as_mut()
      .is_some_and(|clipboard| clipboard.set_text(text).is_ok())
  }

  #[cfg(test)]
  pub(crate) fn memory(text: &str) -> Self {
    Self {
      clipboard: None,
      memory: Some(text.to_string()),
    }
  }

  #[cfg(test)]
  pub(crate) fn unavailable() -> Self {
    Self {
      clipboard: None,
      memory: None,
    }
  }
}
