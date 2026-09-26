//! Terminal service: raw mode / alternate screen lifecycle, capability profile and forced restore.

use std::io::{self, Stdout, Write, stdout};

use crossterm::cursor::{Hide, Show};

use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};

use crossterm::execute;

use crossterm::terminal::{
  EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

mod capabilities;

pub use capabilities::TerminalCapabilities;

/// Terminal service that manages entering and leaving raw mode and the alternate screen.
pub struct TerminalService {
  surface: Option<TerminalSurface>,
  capabilities: TerminalCapabilities,
}

struct TerminalSurface {
  stdout: Stdout,
  active: bool,
}

impl TerminalSurface {
  /// Enters raw mode: enables raw mode, the alternate screen, mouse capture and focus events,
  /// and hides the cursor.
  ///
  /// # Errors
  ///
  /// Returns the I/O error of the first terminal command that fails; a failure after raw mode
  /// was enabled force-restores the terminal first.
  fn enter() -> io::Result<Self> {
    enable_raw_mode()?;

    let mut stdout = stdout();
    let result = (|| {
      execute!(stdout, EnterAlternateScreen)?;
      execute!(stdout, EnableMouseCapture)?;
      execute!(stdout, EnableFocusChange)?;
      execute!(stdout, Hide)?;
      stdout.flush()?;

      Ok(Self {
        stdout,
        active: true,
      })
    })();

    if result.is_err() {
      TerminalService::force_restore();
    }
    result
  }

  fn writer(&mut self) -> &mut Stdout {
    &mut self.stdout
  }

  /// Restores the terminal to normal mode: shows the cursor and disables focus events, mouse
  /// capture, the alternate screen and raw mode. Does nothing when already restored.
  fn restore(&mut self) {
    if !self.active {
      return;
    }

    // TODO: log warning — TerminalSurface does not have access to LogService here,
    // so I/O errors during terminal restore are silently discarded.
    let _ = execute!(self.stdout, Show);
    let _ = execute!(self.stdout, DisableFocusChange);
    let _ = execute!(self.stdout, DisableMouseCapture);
    let _ = execute!(self.stdout, LeaveAlternateScreen);
    let _ = self.stdout.flush();

    let _ = disable_raw_mode();
    let _ = io::stderr().flush();

    self.active = false;
  }
}

impl Drop for TerminalSurface {
  fn drop(&mut self) {
    self.restore();
  }
}

impl TerminalService {
  pub fn new() -> Self {
    Self {
      surface: None,
      capabilities: TerminalCapabilities::detect(),
    }
  }

  pub fn capabilities(&self) -> &TerminalCapabilities {
    &self.capabilities
  }

  pub fn apply_capability_profile(
    &mut self,
    unicode: Option<bool>,
    color: Option<&str>,
    mouse: Option<bool>,
  ) {
    if let Some(unicode) = unicode {
      self.capabilities.unicode = unicode;
    }
    if let Some(color) = color {
      self.capabilities.truecolor = color == "truecolor";
    }
    if let Some(mouse) = mouse {
      self.capabilities.mouse = mouse;
    }
  }

  /// Enters terminal raw mode (enables the alternate screen, mouse capture and focus events).
  ///
  /// Does nothing when the terminal is already entered.
  ///
  /// # Errors
  ///
  /// Returns the I/O error of the first terminal command that fails.
  pub fn enter(&mut self) -> io::Result<()> {
    if self.surface.is_some() {
      return Ok(());
    }

    self.surface = Some(TerminalSurface::enter()?);
    Ok(())
  }

  /// Leaves terminal raw mode.
  pub fn exit(&mut self) {
    self.surface = None;
  }

  pub fn is_active(&self) -> bool {
    self.surface.is_some()
  }

  pub fn writer_mut(&mut self) -> Option<&mut Stdout> {
    self.surface.as_mut().map(|surface| surface.writer())
  }

  /// Clears the screen and moves the cursor to (0, 0). Does nothing when the terminal is not
  /// entered.
  ///
  /// # Errors
  ///
  /// Returns the I/O error when queuing or flushing the terminal commands fails.
  pub fn clear_all_and_home(&mut self) -> io::Result<()> {
    use crossterm::QueueableCommand;
    use crossterm::cursor::MoveTo;
    use crossterm::terminal::{Clear, ClearType};

    if let Some(stdout) = self.writer_mut() {
      stdout.queue(Clear(ClearType::All))?;
      stdout.queue(MoveTo(0, 0))?;
      stdout.flush()?;
    }

    Ok(())
  }

  /// Forcibly restores the terminal settings (cleanup for abnormal exits).
  pub fn force_restore() {
    // TODO: log warning — static method has no access to LogService,
    // so I/O errors during forced terminal restore are silently discarded.
    let _ = disable_raw_mode();

    let mut stdout = stdout();

    let _ = execute!(stdout, Show);
    let _ = execute!(stdout, DisableFocusChange);
    let _ = execute!(stdout, DisableMouseCapture);
    let _ = execute!(stdout, LeaveAlternateScreen);
    let _ = stdout.flush();

    let _ = io::stderr().flush();
  }
}

impl Default for TerminalService {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn capability_profile_updates_runtime_capabilities() {
    let mut terminal = TerminalService::new();

    terminal.apply_capability_profile(Some(false), Some("truecolor"), Some(true));

    assert!(!terminal.capabilities().unicode);
    assert!(terminal.capabilities().truecolor);
    assert!(terminal.capabilities().mouse);
  }
}
