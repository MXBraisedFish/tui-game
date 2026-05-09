//! setting_keybind.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static LIST_GLOBAL: MutableText = MutableText::new();
pub static LIST_SYSTEM: MutableText = MutableText::new();
pub static LIST_GAME: MutableText = MutableText::new();

/// setting_keybind.* 文本集合
#[derive(Clone, Debug)]
pub struct SettingKeybindText {
    pub list_global: String,
    pub list_system: String,
    pub list_game: String,
}

/// 注册 setting_keybind.* 文本
pub fn register(language_source: &LanguageSource) -> SettingKeybindText {
    set_text(&LIST_GLOBAL, language_source, "setting_keybind.list.global");
    set_text(&LIST_SYSTEM, language_source, "setting_keybind.list.system");
    set_text(&LIST_GAME, language_source, "setting_keybind.list.game");

    SettingKeybindText {
        list_global: text(&LIST_GLOBAL),
        list_system: text(&LIST_SYSTEM),
        list_game: text(&LIST_GAME),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
