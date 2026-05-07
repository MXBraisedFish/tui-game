//! mod_security.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static TITLE: MutableText = MutableText::new();
pub static WARN: MutableText = MutableText::new();
pub static MOD: MutableText = MutableText::new();
pub static SECOND: MutableText = MutableText::new();

/// mod_security.* 文本集合
#[derive(Clone, Debug)]
pub struct ModSecurityText {
    pub title: String,
    pub warn: String,
    pub mod_label: String,
    pub second: String,
}

/// 注册 mod_security.* 文本
pub fn register(language_source: &LanguageSource) -> ModSecurityText {
    set_text(&TITLE, language_source, "mod_security.title");
    set_text(&WARN, language_source, "mod_security.warn");
    set_text(&MOD, language_source, "mod_security.mod");
    set_text(&SECOND, language_source, "mod_security.second");

    ModSecurityText {
        title: text(&TITLE),
        warn: text(&WARN),
        mod_label: text(&MOD),
        second: text(&SECOND),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
