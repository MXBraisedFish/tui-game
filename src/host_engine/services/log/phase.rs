#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogPhase {
  Boot,
  Runtime,
  Shutdown,
  Crash,
}

impl LogPhase {
  pub fn key(self) -> &'static str {
    match self {
      Self::Boot => "log.phase.boot",
      Self::Runtime => "log.phase.runtime",
      Self::Shutdown => "log.phase.shutdown",
      Self::Crash => "log.phase.crash",
    }
  }

  pub fn default_label(self) -> &'static str {
    match self {
      Self::Boot => "Boot",
      Self::Runtime => "Runtime",
      Self::Shutdown => "Shutdown",
      Self::Crash => "Crash",
    }
  }
}
