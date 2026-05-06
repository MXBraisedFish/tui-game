//! global.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static ERROR_MISSING_KEY: MutableText = MutableText::new();

/// global.* 文本集合
#[derive(Clone, Debug)]
pub struct GlobalText {
    pub error_missing_key: String,
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

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
