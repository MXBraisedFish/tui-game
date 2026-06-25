use super::{LogEntry, format_log_level};

/// 将一条日志条目格式化为可读字符串（含序号、级别、来源、消息）。
pub fn format_log_entry(entry: &LogEntry) -> String {
  format!(
    "#{:04} [{}] [{:?}] {}",
    entry.sequence,
    format_log_level(entry.level),
    entry.source,
    entry.message,
  )
}
