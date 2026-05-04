//! warning.* 语言文本注册

use once_cell::sync::OnceCell;

use crate::host_engine::boot::i18n::i18n::{resolve_text, LanguageSource};

pub static SIZE_ACTUAL: OnceCell<String> = OnceCell::new();
pub static SIZE_NEEDED: OnceCell<String> = OnceCell::new();
pub static SIZE_HINT: OnceCell<String> = OnceCell::new();
pub static SIZE_ACTION_EXIT: OnceCell<String> = OnceCell::new();
pub static SIZE_ACTION_RETURN: OnceCell<String> = OnceCell::new();

/// warning.* 文本集合
#[derive(Clone, Copy)]
pub struct WarningText {
    pub size_actual: &'static str,
    pub size_needed: &'static str,
    pub size_hint: &'static str,
    pub size_action_exit: &'static str,
    pub size_action_return: &'static str,
}

/// 注册 warning.* 文本
pub fn register(language_source: &LanguageSource) -> WarningText {
    set_text(&SIZE_ACTUAL, language_source, "warning.size.actual");
    set_text(&SIZE_NEEDED, language_source, "warning.size.needed");
    set_text(&SIZE_HINT, language_source, "warning.size.hint");
    set_text(
        &SIZE_ACTION_EXIT,
        language_source,
        "warning.size.action.exit",
    );
    set_text(
        &SIZE_ACTION_RETURN,
        language_source,
        "warning.size.action.return",
    );

    WarningText {
        size_actual: text(&SIZE_ACTUAL),
        size_needed: text(&SIZE_NEEDED),
        size_hint: text(&SIZE_HINT),
        size_action_exit: text(&SIZE_ACTION_EXIT),
        size_action_return: text(&SIZE_ACTION_RETURN),
    }
}

fn set_text(cell: &'static OnceCell<String>, language_source: &LanguageSource, key: &str) {
    let _ = cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static OnceCell<String>) -> &'static str {
    cell.get().map(String::as_str).unwrap_or("")
}
