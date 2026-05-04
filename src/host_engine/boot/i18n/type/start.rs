//! start.* 语言文本注册

use once_cell::sync::OnceCell;

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};

pub static FINISH: OnceCell<String> = OnceCell::new();

/// start.* 文本集合
#[derive(Clone, Copy)]
pub struct StartText {
    pub finish: &'static str,
}

/// 注册 start.* 文本
pub fn register(language_source: &LanguageSource) -> StartText {
    set_text(&FINISH, language_source, "start.finish");

    StartText {
        finish: text(&FINISH),
    }
}

fn set_text(cell: &'static OnceCell<String>, language_source: &LanguageSource, key: &str) {
    let _ = cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static OnceCell<String>) -> &'static str {
    cell.get().map(String::as_str).unwrap_or("")
}
