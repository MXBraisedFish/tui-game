//! Log severity values and stable labels.

/// Log severity ordered from trace through fatal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
  /// The trace setting for log level.
  Trace,
  /// The debug setting for log level.
  Debug,
  /// The info setting for log level.
  Info,
  /// The warn setting for log level.
  Warn,
  /// The error setting for log level.
  Error,
  /// The fatal setting for log level.
  Fatal,
}

/// Return the uppercase label for a log severity.
pub fn format_log_level(level: LogLevel) -> &'static str {
  match level {
    LogLevel::Trace => "TRACE",
    LogLevel::Debug => "DEBUG",
    LogLevel::Info => "INFO",
    LogLevel::Warn => "WARN",
    LogLevel::Error => "ERROR",
    LogLevel::Fatal => "FATAL",
  }
}

impl LogLevel {
  /// Return the stable string key for this log level.
  pub fn key(self) -> &'static str {
    match self {
      Self::Trace => "log.level.trace",
      Self::Debug => "log.level.debug",
      Self::Info => "log.level.info",
      Self::Warn => "log.level.warn",
      Self::Error => "log.level.error",
      Self::Fatal => "log.level.fatal",
    }
  }

  /// Return the embedded English label for this log level.
  pub fn default_label(self) -> &'static str {
    format_log_level(self)
  }
}
