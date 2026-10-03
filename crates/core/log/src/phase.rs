//! Log lifecycle phases and labels.

/// The lifecycle category used in a log header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogPhase {
  /// The boot stage of the operation.
  Boot,
  /// The runtime stage of the operation.
  Runtime,
  /// The shutdown stage of the operation.
  Shutdown,
  /// The crash stage of the operation.
  Crash,
}

impl LogPhase {
  /// Return the stable string key for this log phase.
  pub fn key(self) -> &'static str {
    match self {
      Self::Boot => "log.phase.boot",
      Self::Runtime => "log.phase.runtime",
      Self::Shutdown => "log.phase.shutdown",
      Self::Crash => "log.phase.crash",
    }
  }

  /// Return the embedded English label for this log phase.
  pub fn default_label(self) -> &'static str {
    match self {
      Self::Boot => "Boot",
      Self::Runtime => "Runtime",
      Self::Shutdown => "Shutdown",
      Self::Crash => "Crash",
    }
  }
}
