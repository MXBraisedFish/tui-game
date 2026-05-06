//! setting.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static TITLE: MutableText = MutableText::new();
pub static LANGUAGE: MutableText = MutableText::new();
pub static KEYBIND: MutableText = MutableText::new();
pub static MODS: MutableText = MutableText::new();
pub static MEMORY: MutableText = MutableText::new();
pub static SECURITY: MutableText = MutableText::new();

/// setting.* 文本集合
#[derive(Clone, Debug)]
pub struct SettingText {
    pub title: String,
    pub language: String,
    pub keybind: String,
    pub mods: String,
    pub memory: String,
    pub security: String,
}

/// 注册 setting.* 文本
pub fn register(language_source: &LanguageSource) -> SettingText {
    set_text(&TITLE, language_source, "setting.title");
    set_text(&LANGUAGE, language_source, "setting.language");
    set_text(&KEYBIND, language_source, "setting.keybind");
    set_text(&MODS, language_source, "setting.mods");
    set_text(&MEMORY, language_source, "setting.memory");
    set_text(&SECURITY, language_source, "setting.security");

    SettingText {
        title: text(&TITLE),
        language: text(&LANGUAGE),
        keybind: text(&KEYBIND),
        mods: text(&MODS),
        memory: text(&MEMORY),
        security: text(&SECURITY),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
