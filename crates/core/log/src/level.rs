/// Log severity level, declared in ascending order (Trace is the lowest, Fatal the highest).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
  Trace,
  Debug,
  Info,
  Warn,
  Error,
  Fatal,
}

/// Returns the upper-case label of a log level (such as `"WARN"`).
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

  pub fn default_label(self) -> &'static str {
    format_log_level(self)
  }
}
