//! default_security.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static TITLE: MutableText = MutableText::new();
pub static WARN: MutableText = MutableText::new();
pub static SECOND: MutableText = MutableText::new();

/// default_security.* 文本集合
#[derive(Clone, Debug)]
pub struct DefaultSecurityText {
    pub title: String,
    pub warn: String,
    pub second: String,
}

/// 注册 default_security.* 文本
pub fn register(language_source: &LanguageSource) -> DefaultSecurityText {
    set_text(&TITLE, language_source, "default_security.title");
    set_text(&WARN, language_source, "default_security.warn");
    set_text(&SECOND, language_source, "default_security.second");

    DefaultSecurityText {
        title: text(&TITLE),
        warn: text(&WARN),
        second: text(&SECOND),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
