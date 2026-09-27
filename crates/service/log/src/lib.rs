//! Log service: buffered host/package/Lua session logs with translated labels and file output.

mod formatter;
mod labels;
mod service;

pub use formatter::{format_file_log_entry, format_log_entry, format_print_log_entry};
pub use labels::LogLabels;
pub use service::{LogService, LogSessionId, LogSessionKind};
pub use tg_core_log::{
  HostLogMessage, LogEntry, LogLevel, LogPhase, LogPrintOptions, LogSource, format_log_level,
};
