//! Rust UI framework skeleton.
//!
//! This module is intentionally parallel to the current Lua UI runtime. It provides
//! reusable Rust-side page, canvas and component abstractions without changing the
//! active runtime loop yet.

pub mod canvas;
pub mod components;
pub mod pages;
pub mod ui_manager;

pub use canvas::Canvas;
pub use ui_manager::{UiContext, UiEvent, UiManager, UiNavigation, UiPage, UiResult};

pub type HostI18n = crate::host_engine::boot::i18n::I18nText;
