//! key.* 语言文本注册

use once_cell::sync::OnceCell;

use crate::host_engine::boot::i18n::i18n::{resolve_text, LanguageSource};

pub static HOME_PREV_OPTION: OnceCell<String> = OnceCell::new();
pub static HOME_NEXT_OPTION: OnceCell<String> = OnceCell::new();
pub static HOME_SELECT: OnceCell<String> = OnceCell::new();
pub static HOME_CONFIRM: OnceCell<String> = OnceCell::new();
pub static HOME_OPTION1: OnceCell<String> = OnceCell::new();
pub static HOME_OPTION2: OnceCell<String> = OnceCell::new();
pub static HOME_OPTION3: OnceCell<String> = OnceCell::new();
pub static HOME_OPTION4: OnceCell<String> = OnceCell::new();
pub static HOME_OPTION5: OnceCell<String> = OnceCell::new();

/// key.* 文本集合
#[derive(Clone, Copy)]
pub struct KeyText {
    pub home_prev_option: &'static str,
    pub home_next_option: &'static str,
    pub home_select: &'static str,
    pub home_confirm: &'static str,
    pub home_option1: &'static str,
    pub home_option2: &'static str,
    pub home_option3: &'static str,
    pub home_option4: &'static str,
    pub home_option5: &'static str,
}

/// 注册 key.* 文本
pub fn register(language_source: &LanguageSource) -> KeyText {
    set_text(&HOME_PREV_OPTION, language_source, "key.home.prev_option");
    set_text(&HOME_NEXT_OPTION, language_source, "key.home.next_option");
    set_text(&HOME_SELECT, language_source, "key.home.select");
    set_text(&HOME_CONFIRM, language_source, "key.home.confirm");
    set_text(&HOME_OPTION1, language_source, "key.home.option1");
    set_text(&HOME_OPTION2, language_source, "key.home.option2");
    set_text(&HOME_OPTION3, language_source, "key.home.option3");
    set_text(&HOME_OPTION4, language_source, "key.home.option4");
    set_text(&HOME_OPTION5, language_source, "key.home.option5");

    KeyText {
        home_prev_option: text(&HOME_PREV_OPTION),
        home_next_option: text(&HOME_NEXT_OPTION),
        home_select: text(&HOME_SELECT),
        home_confirm: text(&HOME_CONFIRM),
        home_option1: text(&HOME_OPTION1),
        home_option2: text(&HOME_OPTION2),
        home_option3: text(&HOME_OPTION3),
        home_option4: text(&HOME_OPTION4),
        home_option5: text(&HOME_OPTION5),
    }
}

fn set_text(cell: &'static OnceCell<String>, language_source: &LanguageSource, key: &str) {
    let _ = cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static OnceCell<String>) -> &'static str {
    cell.get().map(String::as_str).unwrap_or("")
}
