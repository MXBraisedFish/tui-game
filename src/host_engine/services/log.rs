// 引入官方标准双端队列
use std::collections::VecDeque;

// 日志等级
#[derive(Clone, Debug)]
pub enum LogLevel {
  Info,
  Warn,
  Error
}

// 日志信息
#[derive(Clone, Debug)]
pub struct LogEntry {
  pub level: LogLevel,
  pub message: String
}

pub struct LogService {
  queue: VecDeque<LogEntry>
}

impl LogService {
  pub fn new() -> Self {
    Self {
      queue: VecDeque::new()
    }
  }

  // 入队信息
  // impl Into<String>是泛型的一种语法糖写法
  // Rust的写法总能给我带来惊喜
  pub fn info(&mut self, message: impl Into<String>) {
    self.push(LogLevel::Info, message);
  }

  // 入队警告
  pub fn warn(&mut self, message: impl Into<String>) {
    self.push(LogLevel::Warn, message);
  }
  
  // 入队错误
  pub fn error(&mut self, message: impl Into<String>) {
    self.push(LogLevel::Error, message);
  }

  // 入队操作
  fn push(&mut self, level: LogLevel, message: impl Into<String>) {
    // 放入队尾
    self.queue.push_back(LogEntry {
      level,
      message: message.into()
    });

    // 如果太长就把队头弹出
    while self.queue.len() > 1000 {
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
    self.queue.drain(..).collect() // 返回Vec
  }
}