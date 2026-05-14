//! mod.list.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static GAME: MutableText = MutableText::new();
pub static SAVER: MutableText = MutableText::new();
pub static BOSS: MutableText = MutableText::new();

#[derive(Clone, Debug)]
pub struct ModHubText {
    pub game: String,
    pub saver: String,
    pub boss: String,
}

pub fn register(language_source: &LanguageSource) -> ModHubText {
    set_text(&GAME, language_source, "mod.list.game");
    set_text(&SAVER, language_source, "mod.list.saver");
    set_text(&BOSS, language_source, "mod.list.boss");

    ModHubText {
        game: text(&GAME),
        saver: text(&SAVER),
        boss: text(&BOSS),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
