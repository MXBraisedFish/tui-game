//! Log severity, source, phase, message templates, and record data.
//!
//! # Examples
//!
//! ```rust
//! use tg_core_log::HostLogMessage;
//!
//! let message = HostLogMessage::new("example", "Value: {value}").param("value", "42");
//! assert_eq!(message.render(None), "Value: 42");
//! ```

mod entry;
mod level;
mod message;
mod phase;
mod source;

pub use entry::{LogEntry, LogPrintOptions};
pub use level::{LogLevel, format_log_level};
pub use message::HostLogMessage;
pub use phase::LogPhase;
pub use source::LogSource;
