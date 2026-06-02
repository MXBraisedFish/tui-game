use super::{LogLevel, LogSource};

// 日志信息
#[derive(Clone, Debug)]
pub struct LogEntry {
  pub timestamp_ms: u128, // 时间戳
  pub sequence: u64,      // 序号
  pub level: LogLevel,    // 等级
  pub source: LogSource,  // 来源
  pub message: String,    // 信息内容
}
