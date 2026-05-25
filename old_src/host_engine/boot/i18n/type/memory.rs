//! memory.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static TITLE: MutableText = MutableText::new();
pub static CACHE: MutableText = MutableText::new();
pub static DATA: MutableText = MutableText::new();
pub static SHOW: MutableText = MutableText::new();
pub static INFO_DIR: MutableText = MutableText::new();
pub static INFO_SIZE: MutableText = MutableText::new();
pub static INFO_PATH: MutableText = MutableText::new();
pub static INFO_NAME_ROOT: MutableText = MutableText::new();
pub static INFO_NAME_DATA: MutableText = MutableText::new();
pub static INFO_NAME_CACHE: MutableText = MutableText::new();
pub static INFO_NAME_PROFILES: MutableText = MutableText::new();
pub static INFO_NAME_LOG: MutableText = MutableText::new();
pub static INFO_NAME_MOD: MutableText = MutableText::new();
pub static TIP: MutableText = MutableText::new();

/// memory.* 文本集合
#[derive(Clone, Debug)]
pub struct MemoryText {
    pub title: String,
    pub cache: String,
    pub data: String,
    pub show: String,
    pub info_dir: String,
    pub info_size: String,
    pub info_path: String,
    pub info_name_root: String,
    pub info_name_data: String,
    pub info_name_cache: String,
    pub info_name_profiles: String,
    pub info_name_log: String,
    pub info_name_mod: String,
    pub tip: String,
}

/// 注册 memory.* 文本
pub fn register(language_source: &LanguageSource) -> MemoryText {
    set_text(&TITLE, language_source, "memory.title");
    set_text(&CACHE, language_source, "memory.cache");
    set_text(&DATA, language_source, "memory.data");
    set_text(&SHOW, language_source, "memory.show");
    set_text(&INFO_DIR, language_source, "memory.info.dir");
    set_text(&INFO_SIZE, language_source, "memory.info.size");
    set_text(&INFO_PATH, language_source, "memory.info.path");
    set_text(&INFO_NAME_ROOT, language_source, "memory.info.name.root");
    set_text(&INFO_NAME_DATA, language_source, "memory.info.name.data");
    set_text(&INFO_NAME_CACHE, language_source, "memory.info.name.cache");
    set_text(
        &INFO_NAME_PROFILES,
        language_source,
        "memory.info.name.profiles",
    );
    set_text(&INFO_NAME_LOG, language_source, "memory.info.name.log");
    set_text(&INFO_NAME_MOD, language_source, "memory.info.name.mod");
    set_text(&TIP, language_source, "memory.tip");

    MemoryText {
        title: text(&TITLE),
        cache: text(&CACHE),
        data: text(&DATA),
        show: text(&SHOW),
        info_dir: text(&INFO_DIR),
        info_size: text(&INFO_SIZE),
        info_path: text(&INFO_PATH),
        info_name_root: text(&INFO_NAME_ROOT),
        info_name_data: text(&INFO_NAME_DATA),
        info_name_cache: text(&INFO_NAME_CACHE),
        info_name_profiles: text(&INFO_NAME_PROFILES),
        info_name_log: text(&INFO_NAME_LOG),
        info_name_mod: text(&INFO_NAME_MOD),
        tip: text(&TIP),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
