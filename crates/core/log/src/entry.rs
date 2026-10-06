//! Timestamped log records and script print-header options.

use super::{LogLevel, LogSource};

/// A timestamped log record with sequence, severity, source, and message.
///
/// # Fields
///
/// * `timestamp_ms` - The timestamp measured in milliseconds.
/// * `sequence` - The sequence.
/// * `level` - The level.
/// * `source` - The log source carried by this log entry.
/// * `message` - The diagnostic or display message.
#[derive(Clone, Debug)]
pub struct LogEntry {
  /// The timestamp measured in milliseconds.
  pub timestamp_ms: u128,
  /// The sequence.
  pub sequence: u64,
  /// The level.
  pub level: LogLevel,
  /// The log source carried by this log entry.
  pub source: LogSource,
  /// The diagnostic or display message.
  pub message: String,
}

/// Optional headers requested by script-generated log output.
///
/// # Fields
///
/// * `time` - Whether the printed log header includes time.
/// * `level` - The level.
/// * `type_head` - Whether the printed log header includes its source category.
/// * `title` - The title.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LogPrintOptions {
  /// Whether the printed log header includes time.
  pub time: bool,
  /// The level.
  pub level: Option<LogLevel>,
  /// Whether the printed log header includes its source category.
  pub type_head: bool,
  /// The title.
  pub title: Option<String>,
}
