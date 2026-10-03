//! Terminal capabilities, raw-mode entry, alternate-screen ownership, and restoration.
//!
//! # Examples
//!
//! ```rust
//! use tg_service_terminal::TerminalService;
//!
//! let mut terminal = TerminalService::new();
//! terminal.apply_capability_profile(Some(true), Some("truecolor"), Some(true));
//! assert!(terminal.capabilities().truecolor);
//! assert!(!terminal.is_active());
//! ```

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

/// The public entry point for terminal operations.
pub struct TerminalService {
  surface: Option<TerminalSurface>,
  capabilities: TerminalCapabilities,
}

struct TerminalSurface {
  stdout: Stdout,
  active: bool,
}

impl TerminalSurface {
  /// Acquire raw mode and alternate-screen ownership, restoring the terminal if partial setup
  /// fails.
  ///
  /// # Errors
  ///
  /// Propagate terminal I/O errors while enabling raw mode, alternate-screen output, and event
  /// capture.
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

  /// Restore cursor, capture, screen, and raw-mode state once, using best-effort cleanup for
  /// Drop.
  fn restore(&mut self) {
    if !self.active {
      return;
    }

    // Restoration is best-effort because Drop cannot propagate terminal I/O failures.

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
  /// Create a terminal service with its initial state.
  pub fn new() -> Self {
    Self {
      surface: None,
      capabilities: TerminalCapabilities::detect(),
    }
  }

  /// Return the current capabilities.
  pub fn capabilities(&self) -> &TerminalCapabilities {
    &self.capabilities
  }

  /// Apply user-selected capability overrides to the detected terminal profile.
  ///
  /// # Arguments
  ///
  /// * `unicode` - Whether Unicode output is supported; `None` keeps the detected setting.
  /// * `color` - The optional terminal color profile; `truecolor` enables full RGB output.
  /// * `mouse` - Whether terminal pointer input is supported; `None` keeps the detected setting.
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

  /// Acquire raw mode and alternate-screen ownership, restoring the terminal if partial setup
  /// fails.
  ///
  /// # Errors
  ///
  /// Propagate terminal I/O errors while enabling raw mode, alternate-screen output, and event
  /// capture.
  pub fn enter(&mut self) -> io::Result<()> {
    if self.surface.is_some() {
      return Ok(());
    }

    self.surface = Some(TerminalSurface::enter()?);
    Ok(())
  }

  /// Restore terminal mode and output state before releasing the active terminal session.
  pub fn exit(&mut self) {
    self.surface = None;
  }

  /// Report whether this terminal service is active.
  pub fn is_active(&self) -> bool {
    self.surface.is_some()
  }

  /// Return mutable access to the owned writer.
  pub fn writer_mut(&mut self) -> Option<&mut Stdout> {
    self.surface.as_mut().map(|surface| surface.writer())
  }

  /// Clear terminal content and move the output cursor to the origin.
  ///
  /// # Errors
  ///
  /// Propagate terminal I/O errors while clearing the screen or moving the cursor.
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

  /// Attempt terminal restoration without requiring a live terminal service instance.
  pub fn force_restore() {
    // Forced restoration is best-effort and has no log-service dependency.

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
