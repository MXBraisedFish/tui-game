//! Directory archives with a manifest, progress events, and cancellation cleanup.
//!
//! # Examples
//!
//! ```rust
//! use tg_service_export::ExportFormat;
//!
//! assert_eq!(ExportFormat::Zip.extension(), "zip");
//! ```

mod service;

pub use service::{ExportAsyncEvent, ExportFormat, ExportScope, ExportService, ExportTask};
