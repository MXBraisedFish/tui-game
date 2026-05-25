//! clear_cache.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static TITLE: MutableText = MutableText::new();
pub static WARN: MutableText = MutableText::new();
pub static CACHE_PATH: MutableText = MutableText::new();
pub static LOG_PATH: MutableText = MutableText::new();
pub static SECOND: MutableText = MutableText::new();

/// clear_cache.* 文本集合
#[derive(Clone, Debug)]
pub struct ClearCacheText {
    pub title: String,
    pub warn: String,
    pub cache_path: String,
    pub log_path: String,
    pub second: String,
}

/// 注册 clear_cache.* 文本
pub fn register(language_source: &LanguageSource) -> ClearCacheText {
    set_text(&TITLE, language_source, "clear_cache.title");
    set_text(&WARN, language_source, "clear_cache.warn");
    set_text(&CACHE_PATH, language_source, "clear_cache.cache_path");
    set_text(&LOG_PATH, language_source, "clear_cache.log_path");
    set_text(&SECOND, language_source, "clear_cache.second");

    ClearCacheText {
        title: text(&TITLE),
        warn: text(&WARN),
        cache_path: text(&CACHE_PATH),
        log_path: text(&LOG_PATH),
        second: text(&SECOND),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
