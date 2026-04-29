//! 预加载阶段：读取官方游戏和第三方游戏模块

pub mod cache;
pub mod manifest;
pub mod scanner;
pub mod source;
pub mod uid;

pub use manifest::{GameActionBinding, GameManifest, GameModule, GameModuleRegistry};
pub use source::GameModuleSource;

type GameModuleResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 扫描并读取所有游戏模块
pub fn load() -> GameModuleResult<GameModuleRegistry> {
    let mut registry = GameModuleRegistry::default();

    let official_modules = scanner::scan_source(GameModuleSource::Office)?;
    let mod_modules = scanner::scan_source(GameModuleSource::Mod)?;

    registry.extend(official_modules);
    registry.extend(mod_modules);

    cache::persist_scan_cache(&registry)?;
    cache::persist_default_keybinds(&registry)?;
    cache::persist_default_mod_state(&registry)?;

    Ok(registry)
}
