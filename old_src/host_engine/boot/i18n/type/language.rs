//! language.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static TITLE: MutableText = MutableText::new();
pub static NAME: MutableText = MutableText::new();

/// language.* 文本集合
#[derive(Clone, Debug)]
pub struct LanguageText {
    pub title: String,
    pub name: String,
}

/// 注册 language.* 文本
pub fn register(language_source: &LanguageSource) -> LanguageText {
    set_text(&TITLE, language_source, "language.title");
    set_text(&NAME, language_source, "language.name");

    LanguageText {
        title: text(&TITLE),
        name: text(&NAME),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
