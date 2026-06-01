// 日志级别枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
  Trace, // 追踪
  Debug, // 调试
  Info, // 信息
  Warn, // 警告
  Error, // 错误
  Fatal // 异常
}

// 等级字符串
pub fn format_log_level(level: LogLevel) -> &'static str {
  // TODO(log/i18n): log level text can be localized later.
  match level {
    LogLevel::Trace => "TRACE",
    LogLevel::Debug => "DEBUG",
    LogLevel::Info => "INFO",
    LogLevel::Warn => "WARN",
    LogLevel::Error => "ERROR",
    LogLevel::Fatal => "FATAL",
  }
}