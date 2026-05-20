//! 预加载阶段：读取官方游戏和第三方游戏模块

pub mod cache;
pub mod manifest;
pub mod scanner;
pub mod source;
pub mod uid;

pub use manifest::{GameActionBinding, GameManifest, GameModule, GameModuleRegistry};
pub use source::GameModuleSource;

use crate::host_engine::package::package_id::PackageId;
use crate::host_engine::package::package_id_registry::PackageIdRegistry;

type GameModuleResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 扫描并读取所有游戏模块
pub fn load() -> GameModuleResult<GameModuleRegistry> {
    let mut registry = GameModuleRegistry::default();

    let official_modules = scanner::scan_source(GameModuleSource::Office)?;
    let mod_modules = scanner::scan_source(GameModuleSource::Mod)?;

    registry.extend(official_modules);
    registry.extend(mod_modules);

    warn_uid_conflicts(&registry);

    cache::persist_default_keybinds(&registry)?;
    cache::persist_default_game_state(&registry)?;

    Ok(registry)
}

fn warn_uid_conflicts(registry: &GameModuleRegistry) {
    let mut package_id_registry = PackageIdRegistry::default();

    for game_module in &registry.games {
        let source = match game_module.source {
            GameModuleSource::Office => "official",
            GameModuleSource::Mod => "mod",
        };
        let package_id = PackageId::from_legacy(source, "game", &game_module.uid);
        if let Err(error) = package_id_registry.register(&package_id) {
            eprintln!("[warning] package uid conflict: {error}");
        }
    }
}
