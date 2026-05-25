//! 游戏模块扫描缓存与持久化默认数据
// TODO: 迁移至 storage::CacheStore

use crate::host_engine::boot::environment::data_dirs;
use std::fs;
use std::path::Path;

use serde_json::{Map, Value, json};

use crate::host_engine::boot::preload::persistent_data::{keybind_profile, security_profile};

use super::manifest::GameModuleRegistry;
use super::source::GameModuleSource;

type CacheResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 将默认按键信息写入持久化按键偏好。已有用户数据不会被覆盖。
pub fn persist_default_keybinds(registry: &GameModuleRegistry) -> CacheResult<()> {
    let path = data_dirs::root_dir().join("data/profiles/keybind.json");
    let mut root = keybind_profile::read_keybind_profile(&path);
    let game_section = root
        .as_object_mut()
        .and_then(|root_object| root_object.get_mut(keybind_profile::GAME_SECTION))
        .and_then(Value::as_object_mut)
        .expect("normalized keybind profile must contain game object");

    for game_module in &registry.games {
        let game_entry = game_section
            .entry(game_module.uid.clone())
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(game_object) = game_entry.as_object_mut() else {
            continue;
        };

        for (action_name, action_binding) in &game_module.game.actions {
            game_object.entry(action_name.clone()).or_insert_with(|| {
                keybind_profile::keybind_entry(&action_binding.key, &action_binding.key_name)
            });
        }
    }

    keybind_profile::write_keybind_profile(&path, &root)
}

/// 将第三方模块默认状态写入 game_state。已有用户状态不会被覆盖。
pub fn persist_default_game_state(registry: &GameModuleRegistry) -> CacheResult<()> {
    let path = data_dirs::root_dir().join("data/profiles/game_state.json");
    let security = security_profile::load_from_default_path();
    let mut root = read_json_object(&path);

    for game_module in &registry.games {
        if game_module.source != GameModuleSource::Mod {
            continue;
        }

        root.entry(game_module.uid.clone()).or_insert_with(|| {
            json!({
                "package": game_module.package.package,
                "enabled": security.default_mod_game_enabled,
                "debug": false,
                "safe_mode": security.default_safe_mode,
                "safe_mode_permanent": !security.default_safe_mode
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
