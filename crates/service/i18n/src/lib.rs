//! Language registries, runtime namespaces, package text, and embedded English fallback.
//!
//! # Examples
//!
//! ```rust
//! use tg_service_i18n::I18nService;
//!
//! fn main() {
//!   let mut i18n = I18nService::new();
//!   i18n.load_embedded_fallback();
//!   let text = i18n.get_runtime_text("exit_warning", "exit_warning.exception.countdown");
//!   assert_eq!(text, "Exiting in {value:second} seconds");
//!   println!("i18n ok: {text}");
//! }
//! ```

mod embedded;
mod language_info;
mod manage;
mod registry;
mod runtime;
mod service;

pub use language_info::LanguageInfo;
pub use registry::{LanguageRegistryEntry, load_language_registry};
pub use service::I18nService;
