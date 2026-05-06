//! 缓存数据快照

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::host_engine::boot::preload::game_modules::GameModuleRegistry;

/// 单个语言文件中供语言选择 UI 使用的文本。
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LanguageUiText {
    pub key_language_up_option: String,
    pub key_language_down_option: String,
    pub key_language_left_option: String,
    pub key_language_right_option: String,
    pub key_language_select: String,
    pub key_language_confirm: String,
    pub key_language_jump: String,
    pub key_language_prev_page: String,
    pub key_language_next_page: String,
    pub key_language_back_cancel: String,
    pub key_language_back: String,
    pub key_language_cancel: String,
    pub key_language_page: String,
    pub key_language_flip: String,
    pub language_title: String,
    pub language_name: String,
}

/// data/cache 下的缓存读取与同步结果
#[derive(Clone, Debug)]
pub struct CacheData {
    pub previous_game_module_registry: GameModuleRegistry,
    pub current_game_module_registry: GameModuleRegistry,
    pub removed_game_uids: Vec<String>,
    pub image_cache_dir: PathBuf,
    pub language_ui_texts: BTreeMap<String, LanguageUiText>,
}
