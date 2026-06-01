use super::{format_log_level, LogEntry};

// 格式化日志字符串
pub fn format_log_entry(entry: &LogEntry) -> String {
  format!(
    "#{:04} [{}] [{:?}] {}",
    entry.sequence,
    format_log_level(entry.level),
    entry.source,
    entry.message,
  )
}