//! 运行缓存预加载与同步入口

mod cache_snapshot;
mod game_package_cache;

pub use cache_snapshot::CacheData;

use crate::host_engine::boot::preload::game_modules::GameModuleRegistry;

/// 读取并更新 data/cache 下的运行缓存。
///
/// 此阶段只处理可重建缓存，不读取 profiles/，不修改持久化用户数据。
pub fn load(
    game_module_registry: &GameModuleRegistry,
) -> Result<CacheData, Box<dyn std::error::Error>> {
    game_package_cache::sync_game_package_cache(game_module_registry)
}
