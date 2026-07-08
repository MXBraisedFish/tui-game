use std::collections::VecDeque;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::host_engine::services::{FileService, I18nService};

use super::{LogEntry, LogLabels, LogLevel, LogSource, format_file_log_entry, format_log_entry};

/// 日志服务：以环形队列存储最近 N 条日志，支持按级别写入与导出。
pub struct LogService {
  queue: VecDeque<LogEntry>,
  next_sequence: u64,
  next_file_sequence: u64,
  max_entries: usize,
  output_path: Option<PathBuf>,
  labels: LogLabels,
  last_file_error: Option<String>,
}

impl LogService {
  pub fn new() -> Self {
    Self {
      queue: VecDeque::new(),
      next_sequence: 0,
      next_file_sequence: 0,
      max_entries: 1000,
      output_path: None,
      labels: LogLabels::new(),
      last_file_error: None,
    }
  }

  pub fn set_output_path(&mut self, path: PathBuf) -> io::Result<()> {
    self.output_path = Some(path);
    self.flush_pending_to_file()
  }

  pub fn refresh_labels_from_i18n(&mut self, i18n: &I18nService) -> io::Result<()> {
    self.labels.refresh_from_i18n(i18n);
    self.flush_pending_to_file()
  }

  /// 记录一条 TRACE 级别日志。
  pub fn trace(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Trace, source, message);
  }

  /// 记录一条 DEBUG 级别日志。
  pub fn debug(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Debug, source, message);
  }

  /// 记录一条 INFO 级别日志。
  pub fn info(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Info, source, message);
  }

  /// 记录一条 WARN 级别日志。
  pub fn warn(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Warn, source, message);
  }

  /// 记录一条 ERROR 级别日志。
  pub fn error(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Error, source, message);
  }

  /// 记录一条 FATAL 级别日志。
  pub fn fatal(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Fatal, source, message);
  }

  fn push(&mut self, level: LogLevel, source: LogSource, message: impl Into<String>) {
    let entry = LogEntry {
      timestamp_ms: now_ms(),
      sequence: self.next_sequence,
      level,
      source,
      message: message.into(),
    };
    self.next_sequence = self.next_sequence.saturating_add(1);

    self.queue.push_back(entry);
    while self.queue.len() > self.max_entries {
      self.queue.pop_front();
    }

    let _ = self.flush_pending_to_file();
  }

  pub fn entries(&self) -> &VecDeque<LogEntry> {
    &self.queue
  }

  /// 取出队列中所有日志并清空。
  pub fn drain(&mut self) -> Vec<LogEntry> {
    self.queue.drain(..).collect()
  }
  pub fn is_empty(&self) -> bool {
    self.queue.is_empty()
  }

  /// 设置最大存储条数（至少为 1），超出时截断旧条目。
  pub fn set_max_entries(&mut self, max_entries: usize) {
    self.max_entries = max_entries.max(1);

    while self.queue.len() > self.max_entries {
      self.queue.pop_front();
    }
  }

  /// 将当前所有日志输出到控制台（stdout）。
  pub fn flush_to_console(&self) {
    for entry in &self.queue {
      let line = format_log_entry(entry);
      if writeln!(std::io::stdout(), "{}", line).is_err() {
        break; // broken pipe, stop
      }
    }
  }

  pub fn flush_pending_to_file(&mut self) -> io::Result<()> {
    let Some(path) = self.output_path.as_ref() else {
      return Ok(());
    };

    let text = self
      .queue
      .iter()
      .filter(|entry| entry.sequence >= self.next_file_sequence)
      .map(|entry| format_file_log_entry(entry, &self.labels))
      .collect::<String>();

    if text.is_empty() {
      return Ok(());
    }

    match FileService::append_text_to(path, &text) {
      Ok(()) => {
        self.next_file_sequence = self
          .queue
          .back()
          .map(|entry| entry.sequence.saturating_add(1))
          .unwrap_or(self.next_file_sequence);
        self.last_file_error = None;
        Ok(())
      }
      Err(error) => {
        self.last_file_error = Some(error.to_string());
        Err(error)
      }
    }
  }

  pub fn last_file_error(&self) -> Option<&str> {
    self.last_file_error.as_deref()
  }
}

impl Default for LogService {
  fn default() -> Self {
    Self::new()
  }
}

// 获取当前 Unix 毫秒时间戳，失败时回退为 0。
fn now_ms() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_millis())
    .unwrap_or(0)
}

#[cfg(test)]
mod tests {
  use std::fs;

  use super::*;

  #[test]
  fn log_service_appends_structured_entries_to_file() {
    let path = std::env::temp_dir().join(format!(
      "tui_game_log_service_{}_{}.log",
      std::process::id(),
      now_ms()
    ));
    let mut log = LogService::new();

    log.set_output_path(path.clone()).unwrap();
    log.info(LogSource::Storage, "storage ready");

    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("[Runtime][Storage]"));
    assert!(text.contains("[INFO] storage ready"));

    let _ = fs::remove_file(path);
  }
}
