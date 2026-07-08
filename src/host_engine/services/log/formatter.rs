use chrono::{DateTime, Local};
use std::time::{Duration, UNIX_EPOCH};

use super::{LogEntry, LogLabels, format_log_level};

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

pub fn format_file_log_entry(entry: &LogEntry, labels: &LogLabels) -> String {
  format!(
    "[{}][{}][{}][{}] {}\n",
    labels.phase(entry.source.phase()),
    labels.source(entry.source),
    format_log_time(entry.timestamp_ms),
    labels.level(entry.level),
    entry.message,
  )
}

fn format_log_time(timestamp_ms: u128) -> String {
  let system_time = UNIX_EPOCH + Duration::from_millis(timestamp_ms.min(u64::MAX as u128) as u64);
  let datetime: DateTime<Local> = system_time.into();
  datetime.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}
