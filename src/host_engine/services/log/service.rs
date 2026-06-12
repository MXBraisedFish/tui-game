use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{format_log_entry, LogEntry, LogLevel, LogSource};

pub struct LogService {
  queue: VecDeque<LogEntry>,
  next_sequence: u64, // 下个序号
  max_entries: usize, // 最大数量
}

impl LogService {
  pub fn new() -> Self {
    Self {
      queue: VecDeque::new(),
      next_sequence: 0,
      max_entries: 1000,
    }
  }

  // 打印追踪
  pub fn trace(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Trace, source, message);
  }

  // 打印调试
  pub fn debug(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Debug, source, message);
  }

  // 打印信息
  pub fn info(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Info, source, message);
  }

  // 打印警告
  pub fn warn(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Warn, source, message);
  }

  // 打印错误
  pub fn error(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Error, source, message);
  }

  // 打印异常
  pub fn fatal(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Fatal, source, message);
  }

  // 入队列
  fn push(&mut self, level: LogLevel, source: LogSource, message: impl Into<String>) {
    // 构建信息
    let entry = LogEntry {
      timestamp_ms: now_ms(),
      sequence: self.next_sequence,
      level,
      source,
      message: message.into(),
    };

    // 下个序号（安全+1，饱和加法）
    self.next_sequence = self.next_sequence.saturating_add(1);
    // 入队列
    self.queue.push_back(entry);

    // 若长度超过则弹出队头
    while self.queue.len() > self.max_entries {
      self.queue.pop_front();
    }
  }

  // 查看队列
  pub fn entries(&self) -> &VecDeque<LogEntry> {
    &self.queue
  }

  // 取出并清空队列
  pub fn drain(&mut self) -> Vec<LogEntry> {
    // 变成迭代器，清空队列内容
    self.queue.drain(..).collect()
  }

  // 判空
  pub fn is_empty(&self) -> bool {
    self.queue.is_empty()
  }

  // 设置最多数量
  pub fn set_max_entries(&mut self, max_entries: usize) {
    self.max_entries = max_entries.max(1);

    while self.queue.len() > self.max_entries {
      self.queue.pop_front();
    }
  }

  // 打印
  pub fn flush_to_console(&self) {
    // TODO(log): 增加文件输出与控制台输出的双重保证。
    // TODO(log): 增加异步输出，避免阻塞主运行时。
    for entry in &self.queue {
      println!("{}", format_log_entry(entry));
    }
  }
}

fn now_ms() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_millis())
    .unwrap_or(0)
}
