use super::{LogLevel, LogSource};

/// 单条日志记录，包含时间戳、序号、级别、来源和消息文本。
#[derive(Clone, Debug)]
pub struct LogEntry {
  pub timestamp_ms: u128,
  pub sequence: u64,
  pub level: LogLevel,
  pub source: LogSource,
  pub message: String,
}
