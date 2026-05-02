//! global.* 语言文本注册

use once_cell::sync::OnceCell;

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};

pub static ERROR_MISSING_KEY: OnceCell<String> = OnceCell::new();

/// global.* 文本集合
#[derive(Clone, Copy)]
pub struct GlobalText {
    pub error_missing_key: &'static str,
}

/// 注册 global.* 文本
pub fn register(language_source: &LanguageSource) -> GlobalText {
    set_text(
        &ERROR_MISSING_KEY,
        language_source,
        "global.error.missing_key",
    );

    GlobalText {
        error_missing_key: text(&ERROR_MISSING_KEY),
    }
}

fn set_text(cell: &'static OnceCell<String>, language_source: &LanguageSource, key: &str) {
    let _ = cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static OnceCell<String>) -> &'static str {
    cell.get().map(String::as_str).unwrap_or("")
}
