//! warning.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static SIZE_ACTUAL: MutableText = MutableText::new();
pub static SIZE_NEEDED: MutableText = MutableText::new();
pub static SIZE_HINT: MutableText = MutableText::new();
pub static SIZE_ACTION_EXIT: MutableText = MutableText::new();
pub static SIZE_ACTION_RETURN: MutableText = MutableText::new();

/// warning.* 文本集合
#[derive(Clone, Debug)]
pub struct WarningText {
    pub size_actual: String,
    pub size_needed: String,
    pub size_hint: String,
    pub size_action_exit: String,
    pub size_action_return: String,
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

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
