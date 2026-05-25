//! home.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static PLAY: MutableText = MutableText::new();
pub static CONTINUE: MutableText = MutableText::new();
pub static SETTINGS: MutableText = MutableText::new();
pub static ABOUT: MutableText = MutableText::new();
pub static QUIT: MutableText = MutableText::new();

/// home.* 文本集合
#[derive(Clone, Debug)]
pub struct HomeText {
    pub play: String,
    pub continue_game: String,
    pub settings: String,
    pub about: String,
    pub quit: String,
}

/// 注册 home.* 文本
pub fn register(language_source: &LanguageSource) -> HomeText {
    set_text(&PLAY, language_source, "home.play");
    set_text(&CONTINUE, language_source, "home.continue");
    set_text(&SETTINGS, language_source, "home.settings");
    set_text(&ABOUT, language_source, "home.about");
    set_text(&QUIT, language_source, "home.quit");

    HomeText {
        play: text(&PLAY),
        continue_game: text(&CONTINUE),
        settings: text(&SETTINGS),
        about: text(&ABOUT),
        quit: text(&QUIT),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
