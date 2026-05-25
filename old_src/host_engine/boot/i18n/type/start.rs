//! start.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static FINISH: MutableText = MutableText::new();

/// start.* 文本集合
#[derive(Clone, Debug)]
pub struct StartText {
    pub finish: String,
}

/// 注册 start.* 文本
pub fn register(language_source: &LanguageSource) -> StartText {
    set_text(&FINISH, language_source, "start.finish");

    StartText {
        finish: text(&FINISH),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
