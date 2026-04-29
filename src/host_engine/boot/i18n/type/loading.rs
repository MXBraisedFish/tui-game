//! loading.* 语言文本注册

use once_cell::sync::OnceCell;

use crate::host_engine::boot::i18n::i18n::{resolve_text, LanguageSource};

pub static INIT_ENV: OnceCell<String> = OnceCell::new();
pub static SCAN_GAME: OnceCell<String> = OnceCell::new();
pub static SCAN_UI: OnceCell<String> = OnceCell::new();
pub static READ_DATA: OnceCell<String> = OnceCell::new();
pub static PRE_CACHE: OnceCell<String> = OnceCell::new();
pub static READY_LAUNCH: OnceCell<String> = OnceCell::new();
pub static COMPLETE: OnceCell<String> = OnceCell::new();

/// loading.* 文本集合
#[derive(Clone, Copy)]
pub struct LoadingText {
    pub init_env: &'static str,
    pub scan_game: &'static str,
    pub scan_ui: &'static str,
    pub read_data: &'static str,
    pub pre_cache: &'static str,
    pub ready_launch: &'static str,
    pub complete: &'static str,
}

/// 注册 loading.* 文本
pub fn register(language_source: &LanguageSource) -> LoadingText {
    set_text(&INIT_ENV, language_source, "loading.init_env");
    set_text(&SCAN_GAME, language_source, "loading.scan_game");
    set_text(&SCAN_UI, language_source, "loading.scan_ui");
    set_text(&READ_DATA, language_source, "loading.read_data");
    set_text(&PRE_CACHE, language_source, "loading.pre_cache");
    set_text(&READY_LAUNCH, language_source, "loading.ready_launch");
    set_text(&COMPLETE, language_source, "loading.complete");

    LoadingText {
        init_env: text(&INIT_ENV),
        scan_game: text(&SCAN_GAME),
        scan_ui: text(&SCAN_UI),
        read_data: text(&READ_DATA),
        pre_cache: text(&PRE_CACHE),
        ready_launch: text(&READY_LAUNCH),
        complete: text(&COMPLETE),
    }
}

fn set_text(cell: &'static OnceCell<String>, language_source: &LanguageSource, key: &str) {
    let _ = cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static OnceCell<String>) -> &'static str {
    cell.get().map(String::as_str).unwrap_or("")
}
