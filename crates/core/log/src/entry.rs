use super::{LogLevel, LogSource};

/// Single log record with its timestamp, sequence number, level, source and message text.
#[derive(Clone, Debug)]
pub struct LogEntry {
  pub timestamp_ms: u128,
  pub sequence: u64,
  pub level: LogLevel,
  pub source: LogSource,
  pub message: String,
}

/// Optional log header settings, mainly used by scripts for custom printing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LogPrintOptions {
  pub time: bool,
  pub level: Option<LogLevel>,
  pub type_head: bool,
  pub title: Option<String>,
}
