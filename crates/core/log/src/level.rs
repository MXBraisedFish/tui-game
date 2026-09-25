/// 日志严重级别，按升序排列（Trace 最低，Fatal 最高）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
  Trace,
  Debug,
  Info,
  Warn,
  Error,
  Fatal,
}

/// 将日志级别转为固定宽度的大写字符串标识。
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
