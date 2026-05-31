// 引入官方标准双端队列
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

// 日志等级
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
  Trace, // 追踪
  Debug, // 调试
  Info, // 信息
  Warn, // 警告
  Error, // 错误
  Fatal // 异常
}

// 日志来源
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogSource {
  Engine, // 引擎
  Boot, // 启动阶段
  Runtime, // 运行阶段
  Shutdown, // 关闭阶段
  Termianl, // 终端服务
  Render, // 绘制服务
  Input, // 输入服务
  Storage, // 数据服务
  Pack, // 包服务
  Lua, // Lua服务
  Game, // 游戏服务
  Overlay, // 覆盖屏幕服务
  Ui, // 引擎UI服务
  Crash // 异常恢复服务
}

// 日志信息
#[derive(Clone, Debug)]
pub struct LogEntry {
  pub timestamp_ms: u128, // 时间戳
  pub sequence: u64, // 序列号
  pub level: LogLevel, // 等级
  pub source: LogSource, // 来源
  pub message: String // 信息
}

pub struct LogService {
  queue: VecDeque<LogEntry>, // 日志双端队列
  next_sequence: u64, // 下一个日志序号
  max_entries: usize  // 最大项
}

impl LogService {
  pub fn new() -> Self {
    Self {
      queue: VecDeque::new(),
      next_sequence: 0,
      max_entries: 1000
    }
  }

  // impl Into<String>是泛型的一种语法糖写法
  // Rust的写法总能给我带来惊喜
  // 入队追踪
  pub fn trace(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Trace, source, message);
  }

  // 入队调试
  pub fn debug(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Debug, source, message);
  }

  // 入队信息
  pub fn info(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Info, source, message);
  }

  // 入队警告
  pub fn warn(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Warn, source, message);
  }
  
  // 入队错误
  pub fn error(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Error, source, message);
  }
  
  // 入队异常
  pub fn fatal(&mut self, source: LogSource, message: impl Into<String>) {
    self.push(LogLevel::Fatal, source, message);
  }

  // 入队操作
  fn push(&mut self, level: LogLevel, source: LogSource, message: impl Into<String>) {
    // 日志信息
    let entry = LogEntry {
      timestamp_ms: now_ms(),
      sequence: self.next_sequence,
      level,
      source,
      message: message.into()
    };

    // 饱和加法（当数字达到上限时候不再继续增加防止溢出）
    self.next_sequence = self.next_sequence.saturating_add(1);

    // 放入队尾
    self.queue.push_back(entry);

    // 如果太长就把队头弹出
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
    self.queue.drain(..).collect() // 返回Vec
  }

  // 日志队列长度
  pub fn is_empty(&self) -> bool {
    self.queue.is_empty()
  }

  // 设置日志队列最大长度
  pub fn set_max_entries(&mut self, max_entries: usize) {
    // 保证最大长度为1
    self.max_entries = max_entries.max(1);

    // 如果缩小了队列就从队头开始清理
    while self.queue.len() > self.max_entries {
      self.queue.pop_front();
    }
  }

  pub fn flush_to_console(&self) {
    for entry in & self.queue {
      println!("{}", format_log_entry(entry));
    }
  }
}

// 格式化日志字符串
pub fn format_log_entry(entry: &LogEntry) -> String {
  format!(
    "#{:04} [{}] [{:?}] {}",
    entry.sequence,
    format_level(entry.level),
    entry.source,
    entry.message
  )
}

// 等级文本
fn format_level(level: LogLevel) -> &'static str {
  // TODO: 这里需要国际化
  match level {
    LogLevel::Trace => "TRACE",
    LogLevel::Debug => "DEBUG",
    LogLevel::Info => "INFO",
    LogLevel::Warn => "WARN",
    LogLevel::Error => "ERROR",
    LogLevel::Fatal => "FATAL"
  }
}

// 当前时间时间戳转换
fn now_ms() -> u128 {
  SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_millis()).unwrap_or(0)
}