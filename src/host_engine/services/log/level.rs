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
