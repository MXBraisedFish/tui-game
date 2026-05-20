//! 缓存数据快照

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::host_engine::boot::preload::game_modules::GameModuleRegistry;
pub use crate::host_engine::storage::types::LanguageUiText;

/// data/cache 下的缓存读取与同步结果
#[derive(Clone, Debug)]
pub struct CacheData {
    pub previous_game_module_registry: GameModuleRegistry,
    pub current_game_module_registry: GameModuleRegistry,
    pub removed_game_uids: Vec<String>,
    pub image_cache_dir: PathBuf,
    pub language_ui_texts: BTreeMap<String, LanguageUiText>,
}
