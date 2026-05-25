//! error.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static UNKNOWN_COMMAND: MutableText = MutableText::new();
pub static UNKNOWN_PARAMETER: MutableText = MutableText::new();

/// error.* 文本集合
#[derive(Clone, Debug)]
pub struct ErrorText {
    pub unknown_command: String,
    pub unknown_parameter: String,
}

/// 注册 error.* 文本
pub fn register(language_source: &LanguageSource) -> ErrorText {
    set_text(&UNKNOWN_COMMAND, language_source, "error.unknown_command");
    set_text(
        &UNKNOWN_PARAMETER,
        language_source,
        "error.unknown_parameter",
    );

    ErrorText {
        unknown_command: text(&UNKNOWN_COMMAND),
        unknown_parameter: text(&UNKNOWN_PARAMETER),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
