//! loading.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static INIT_ENV: MutableText = MutableText::new();
pub static SCAN_MOD: MutableText = MutableText::new();
pub static SCAN_UI: MutableText = MutableText::new();
pub static READ_DATA: MutableText = MutableText::new();
pub static PRE_CACHE: MutableText = MutableText::new();
pub static READY_LAUNCH: MutableText = MutableText::new();
pub static COMPLETE: MutableText = MutableText::new();

/// loading.* 文本集合
#[derive(Clone, Debug)]
pub struct LoadingText {
    pub init_env: String,
    pub scan_mod: String,
    pub scan_ui: String,
    pub read_data: String,
    pub pre_cache: String,
    pub ready_launch: String,
    pub complete: String,
}

/// 注册 loading.* 文本
pub fn register(language_source: &LanguageSource) -> LoadingText {
    set_text(&INIT_ENV, language_source, "loading.init_env");
    set_text(&SCAN_MOD, language_source, "loading.scan_mod");
    set_text(&SCAN_UI, language_source, "loading.scan_ui");
    set_text(&READ_DATA, language_source, "loading.read_data");
    set_text(&PRE_CACHE, language_source, "loading.pre_cache");
    set_text(&READY_LAUNCH, language_source, "loading.ready_launch");
    set_text(&COMPLETE, language_source, "loading.complete");

    LoadingText {
        init_env: text(&INIT_ENV),
        scan_mod: text(&SCAN_MOD),
        scan_ui: text(&SCAN_UI),
        read_data: text(&READ_DATA),
        pre_cache: text(&PRE_CACHE),
        ready_launch: text(&READY_LAUNCH),
        complete: text(&COMPLETE),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
