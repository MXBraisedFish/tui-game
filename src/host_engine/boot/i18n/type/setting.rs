//! setting.* 语言文本注册

use once_cell::sync::OnceCell;

use crate::host_engine::boot::i18n::i18n::{resolve_text, LanguageSource};

pub static LANGUAGE: OnceCell<String> = OnceCell::new();
pub static KEYBIND: OnceCell<String> = OnceCell::new();
pub static MODS: OnceCell<String> = OnceCell::new();
pub static MEMORY: OnceCell<String> = OnceCell::new();
pub static SECURITY: OnceCell<String> = OnceCell::new();

/// setting.* 文本集合
#[derive(Clone, Copy)]
pub struct SettingText {
    pub language: &'static str,
    pub keybind: &'static str,
    pub mods: &'static str,
    pub memory: &'static str,
    pub security: &'static str,
}

/// 注册 setting.* 文本
pub fn register(language_source: &LanguageSource) -> SettingText {
    set_text(&LANGUAGE, language_source, "setting.language");
    set_text(&KEYBIND, language_source, "setting.keybind");
    set_text(&MODS, language_source, "setting.mods");
    set_text(&MEMORY, language_source, "setting.memory");
    set_text(&SECURITY, language_source, "setting.security");

    SettingText {
        language: text(&LANGUAGE),
        keybind: text(&KEYBIND),
        mods: text(&MODS),
        memory: text(&MEMORY),
        security: text(&SECURITY),
    }
}

fn set_text(cell: &'static OnceCell<String>, language_source: &LanguageSource, key: &str) {
    let _ = cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static OnceCell<String>) -> &'static str {
    cell.get().map(String::as_str).unwrap_or("")
}
