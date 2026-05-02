//! 游戏模块扫描缓存与持久化默认数据

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use super::manifest::GameModuleRegistry;
use super::source::GameModuleSource;

type CacheResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 写入游戏模块扫描缓存
pub fn persist_scan_cache(registry: &GameModuleRegistry) -> CacheResult<()> {
    write_json_pretty(&root_dir().join("data/cache/mod_scan_cache.json"), registry)
}

/// 将默认按键信息写入持久化按键偏好。已有用户数据不会被覆盖。
pub fn persist_default_keybinds(registry: &GameModuleRegistry) -> CacheResult<()> {
    let path = root_dir().join("data/profiles/keybind.json");
    let mut root = read_json_object(&path);

    for game_module in &registry.games {
        let game_entry = root
            .entry(game_module.uid.clone())
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(game_object) = game_entry.as_object_mut() else {
            continue;
        };

        for (action_name, action_binding) in &game_module.game.actions {
            game_object.entry(action_name.clone()).or_insert_with(|| {
                json!({
                    "key": action_binding.key,
                    "key_name": action_binding.key_name,
                    "key_user": action_binding.key
                })
            });
        }
    }

    write_json_pretty(&path, &root)
}

/// 将第三方模块默认状态写入 mod_state。已有用户状态不会被覆盖。
pub fn persist_default_mod_state(registry: &GameModuleRegistry) -> CacheResult<()> {
    let path = root_dir().join("data/profiles/mod_state.json");
    let mut root = read_json_object(&path);

    for game_module in &registry.games {
        if game_module.source != GameModuleSource::Mod {
            continue;
        }

        root.entry(game_module.uid.clone()).or_insert_with(|| {
            json!({
                "package": game_module.package.package,
                "enabled": true,
                "debug": false,
                "safe_mode": true
            })
        });
    }

    write_json_pretty(&path, &root)
}

fn read_json_object(path: &Path) -> Map<String, Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw_json| serde_json::from_str::<Value>(&raw_json).ok())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default()
}

fn write_json_pretty<T: serde::Serialize>(path: &Path, value: &T) -> CacheResult<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

fn root_dir() -> PathBuf {
    std::env::current_dir()
        .ok()
        .filter(|path| path.join("assets").exists() || path.join("Cargo.toml").exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(PathBuf::from))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}
