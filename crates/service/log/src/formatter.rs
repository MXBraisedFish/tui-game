use chrono::{DateTime, Local};
use std::time::{Duration, UNIX_EPOCH};

use super::{LogEntry, LogLabels, LogPrintOptions, format_log_level};

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

pub fn format_print_log_entry(
  entry: &LogEntry,
  labels: &LogLabels,
  options: LogPrintOptions,
) -> String {
  let mut text = String::new();
  if options.type_head {
    text.push('[');
    text.push_str(labels.source(entry.source));
    text.push(']');
  }
  if options.time {
    text.push('[');
    text.push_str(&format_log_time(entry.timestamp_ms));
    text.push(']');
  }
  if let Some(level) = options.level {
    text.push('[');
    text.push_str(labels.level(level));
    text.push(']');
  }
  if let Some(title) = options.title.as_deref() {
    text.push('[');
    text.push_str(title);
    text.push(']');
  }
  if !text.is_empty() {
    text.push(' ');
  }
  text.push_str(&entry.message);
  text.push('\n');
  text
}

fn format_log_time(timestamp_ms: u128) -> String {
  let system_time = UNIX_EPOCH + Duration::from_millis(timestamp_ms.min(u64::MAX as u128) as u64);
  let datetime: DateTime<Local> = system_time.into();
  datetime.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{LogLevel, LogSource};

  #[test]
  fn custom_print_has_no_header_by_default() {
    let entry = LogEntry {
      timestamp_ms: 0,
      sequence: 1,
      level: LogLevel::Info,
      source: LogSource::Lua,
      message: "plain message".to_string(),
    };
    assert_eq!(
      format_print_log_entry(&entry, &LogLabels::new(), LogPrintOptions::default()),
      "plain message\n"
    );
  }

  #[test]
  fn custom_print_adds_only_requested_headers() {
    let entry = LogEntry {
      timestamp_ms: 0,
      sequence: 1,
      level: LogLevel::Warn,
      source: LogSource::Lua,
      message: "message".to_string(),
    };
    let text = format_print_log_entry(
      &entry,
      &LogLabels::new(),
      LogPrintOptions {
        time: false,
        level: Some(LogLevel::Warn),
        type_head: true,
        title: None,
      },
    );
    assert_eq!(text, "[Lua][WARN] message\n");
  }

  #[test]
  fn custom_print_orders_type_time_level_and_title() {
    let entry = LogEntry {
      timestamp_ms: 0,
      sequence: 1,
      level: LogLevel::Info,
      source: LogSource::Game,
      message: "message".to_string(),
    };
    let text = format_print_log_entry(
      &entry,
      &LogLabels::new(),
      LogPrintOptions {
        time: true,
        level: Some(LogLevel::Info),
        type_head: true,
        title: Some("Title".to_string()),
      },
    );
    assert!(text.starts_with("[Game]["));
    assert!(text.ends_with("][INFO][Title] message\n"));
  }
}
