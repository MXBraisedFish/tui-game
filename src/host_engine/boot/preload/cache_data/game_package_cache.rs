//! 游戏包缓存读取、拼合与清理

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::host_engine::boot::preload::game_modules::GameModuleRegistry;

use super::cache_snapshot::CacheData;

type CacheResult<T> = Result<T, Box<dyn std::error::Error>>;

const GAME_SCAN_CACHE_FILE: &str = "mod_scan_cache.json";
const IMAGE_CACHE_DIR: &str = "images";

/// 同步游戏包扫描缓存。
pub fn sync_game_package_cache(
    game_module_registry: &GameModuleRegistry,
) -> CacheResult<CacheData> {
    let cache_dir = root_dir().join("data/cache");
    let image_cache_dir = cache_dir.join(IMAGE_CACHE_DIR);
    let scan_cache_path = cache_dir.join(GAME_SCAN_CACHE_FILE);

    fs::create_dir_all(&cache_dir)?;
    fs::create_dir_all(&image_cache_dir)?;

    let previous_game_module_registry = read_game_scan_cache(&scan_cache_path);
    let removed_game_uids =
        find_removed_game_uids(&previous_game_module_registry, game_module_registry);

    remove_unused_game_cache(&image_cache_dir, &removed_game_uids)?;
    write_game_scan_cache(&scan_cache_path, game_module_registry)?;

    Ok(CacheData {
        previous_game_module_registry,
        current_game_module_registry: game_module_registry.clone(),
        removed_game_uids,
        image_cache_dir,
    })
}

fn read_game_scan_cache(path: &Path) -> GameModuleRegistry {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw_json| serde_json::from_str::<GameModuleRegistry>(&raw_json).ok())
        .unwrap_or_default()
}

fn write_game_scan_cache(
    path: &Path,
    game_module_registry: &GameModuleRegistry,
) -> CacheResult<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(path, serde_json::to_string_pretty(game_module_registry)?)?;
    Ok(())
}

fn find_removed_game_uids(
    previous_game_module_registry: &GameModuleRegistry,
    current_game_module_registry: &GameModuleRegistry,
) -> Vec<String> {
    let current_game_uids = current_game_module_registry
        .games
        .iter()
        .map(|game_module| game_module.uid.as_str())
        .collect::<HashSet<_>>();

    previous_game_module_registry
        .games
        .iter()
        .filter(|game_module| !current_game_uids.contains(game_module.uid.as_str()))
        .map(|game_module| game_module.uid.clone())
        .collect()
}

fn remove_unused_game_cache(
    image_cache_dir: &Path,
    removed_game_uids: &[String],
) -> CacheResult<()> {
    if removed_game_uids.is_empty() || !image_cache_dir.is_dir() {
        return Ok(());
    }

    let removed_game_uid_set = removed_game_uids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    for entry in fs::read_dir(image_cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) else {
            continue;
        };

        if !is_removed_game_cache_name(file_name, &removed_game_uid_set) {
            continue;
        }

        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else if path.is_file() {
            fs::remove_file(&path)?;
        }
    }

    Ok(())
}

fn is_removed_game_cache_name(file_name: &str, removed_game_uid_set: &HashSet<&str>) -> bool {
    removed_game_uid_set.iter().any(|removed_game_uid| {
        file_name == *removed_game_uid || file_name.starts_with(&format!("{removed_game_uid}."))
    })
}

fn root_dir() -> PathBuf {
    std::env::current_dir()
        .ok()
        .filter(|path| path.join("assets").exists() || path.join("Cargo.toml").exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(Path::to_path_buf))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}
