//! 运行缓存预加载与同步入口
// TODO: 迁移至 storage::CacheStore

mod cache_snapshot;
mod game_package_cache;
mod image_cache;
mod language_ui_cache;

pub use cache_snapshot::{CacheData, LanguageUiText};

use crate::host_engine::boot::preload::game_modules::GameModuleRegistry;
use crate::host_engine::storage::cache_store::CacheStore;

/// 读取并更新 data/cache 下的运行缓存。
///
/// 此阶段只处理可重建缓存，不读取 profiles/，不修改持久化用户数据。
pub fn load(
    game_module_registry: &GameModuleRegistry,
) -> Result<CacheData, Box<dyn std::error::Error>> {
    let cache_store = CacheStore::open()?;
    let mut cache_data =
        game_package_cache::sync_game_package_cache(&cache_store, game_module_registry)?;
    cache_data.language_ui_texts = language_ui_cache::sync_language_ui_cache(&cache_store)?;
    Ok(cache_data)
}
