use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{LogEntry, LogLevel, LogSource, format_log_entry};

/// 日志服务：以环形队列存储最近 N 条日志，支持按级别写入与导出。
pub struct LogService {
  queue: VecDeque<LogEntry>,
  next_sequence: u64,
  max_entries: usize,
}

impl LogService {
  pub fn new() -> Self {
    Self {
      queue: VecDeque::new(),
      next_sequence: 0,
      max_entries: 1000,
    }
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
      println!("{}", format_log_entry(entry));
    }
  }
}

// 获取当前 Unix 毫秒时间戳，失败时回退为 0。
fn now_ms() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_millis())
    .unwrap_or(0)
}
