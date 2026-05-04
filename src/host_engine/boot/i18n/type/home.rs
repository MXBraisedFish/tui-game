//! home.* 语言文本注册

use once_cell::sync::OnceCell;

use crate::host_engine::boot::i18n::i18n::{resolve_text, LanguageSource};

pub static PLAY: OnceCell<String> = OnceCell::new();
pub static CONTINUE: OnceCell<String> = OnceCell::new();
pub static SETTINGS: OnceCell<String> = OnceCell::new();
pub static ABOUT: OnceCell<String> = OnceCell::new();
pub static QUIT: OnceCell<String> = OnceCell::new();

/// home.* 文本集合
#[derive(Clone, Copy)]
pub struct HomeText {
    pub play: &'static str,
    pub continue_game: &'static str,
    pub settings: &'static str,
    pub about: &'static str,
    pub quit: &'static str,
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

fn set_text(cell: &'static OnceCell<String>, language_source: &LanguageSource, key: &str) {
    let _ = cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static OnceCell<String>) -> &'static str {
    cell.get().map(String::as_str).unwrap_or("")
}
