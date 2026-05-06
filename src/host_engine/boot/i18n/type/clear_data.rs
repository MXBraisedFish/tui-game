//! clear_data.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static TITLE: MutableText = MutableText::new();
pub static WARN: MutableText = MutableText::new();
pub static PATH: MutableText = MutableText::new();
pub static SECOND: MutableText = MutableText::new();

/// clear_data.* 文本集合
#[derive(Clone, Debug)]
pub struct ClearDataText {
    pub title: String,
    pub warn: String,
    pub path: String,
    pub second: String,
}

/// 注册 clear_data.* 文本
pub fn register(language_source: &LanguageSource) -> ClearDataText {
    set_text(&TITLE, language_source, "clear_data.title");
    set_text(&WARN, language_source, "clear_data.warn");
    set_text(&PATH, language_source, "clear_data.path");
    set_text(&SECOND, language_source, "clear_data.second");

    ClearDataText {
        title: text(&TITLE),
        warn: text(&WARN),
        path: text(&PATH),
        second: text(&SECOND),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
