//! Localized log formatting, session routing, bounded history, and file output.
//!
//! # Examples
//!
//! ```rust
//! use std::collections::HashMap;
//!
//! use tg_core_log::{HostLogMessage, LogSource};
//! use tg_service_log::{LogService, format_log_entry};
//!
//! fn main() {
//!   let mut log = LogService::new();
//!   let templates = HashMap::from([("log.smoke".to_string(), "hello {name}".to_string())]);
//!   log
//!     .refresh_labels(
//!       |key| (key == "log.service.engine").then(|| "Engine*".to_string()),
//!       templates,
//!     )
//!     .expect("no file output configured");
//!   log.info_message(
//!     LogSource::Engine,
//!     HostLogMessage::new("log.smoke", "fallback {name}").param("name", "tg"),
//!   );
//!   let entry = log.entries().back().expect("one entry");
//!   assert_eq!(entry.message, "hello tg");
//!   println!("log ok: {}", format_log_entry(entry));
//! }
//! ```

mod formatter;
mod labels;
mod service;

pub use formatter::{format_file_log_entry, format_log_entry, format_print_log_entry};
pub use labels::LogLabels;
pub use service::{LogService, LogSessionId, LogSessionKind};
pub use tg_core_log::{
  HostLogMessage, LogEntry, LogLevel, LogPhase, LogPrintOptions, LogSource, format_log_level,
};
