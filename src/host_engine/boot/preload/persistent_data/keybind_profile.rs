//! 按键偏好持久化结构。

use std::fs;
use std::path::Path;

use serde_json::{Map, Value, json};

use crate::host_engine::constant::MAX_ACTION_KEYS;

pub const GLOBAL_SECTION: &str = "global";
pub const SYSTEM_SECTION: &str = "system";
pub const GAME_SECTION: &str = "game";

type KeybindResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 读取并规范化 keybind.json。
pub fn load_keybind_profile(path: &Path) -> KeybindResult<Value> {
    let mut keybinds = read_json_object(path)?;
    let normalized_keybinds = normalize_keybind_profile(std::mem::take(&mut keybinds));
    write_json_pretty(path, &normalized_keybinds)?;
    Ok(normalized_keybinds)
}

/// 读取 keybind.json，用于默认值持久化。
pub fn read_keybind_profile(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw_json| serde_json::from_str::<Value>(&raw_json).ok())
        .map(normalize_keybind_profile)
        .unwrap_or_else(empty_keybind_profile)
}

/// 写入 keybind.json。
pub fn write_keybind_profile(path: &Path, keybinds: &Value) -> KeybindResult<()> {
    write_json_pretty(path, keybinds)
}

/// 获取 game 分区。
pub fn game_section(keybinds: &Value) -> Option<&Map<String, Value>> {
    keybinds.get(GAME_SECTION).and_then(Value::as_object)
}

/// 获取 system 分区。
pub fn system_section(keybinds: &Value) -> Option<&Map<String, Value>> {
    keybinds.get(SYSTEM_SECTION).and_then(Value::as_object)
}

/// 构造按键绑定项。
pub fn keybind_entry(key: &Value, key_name: &str) -> Value {
    json!({
        "key": truncate_key_value(key),
        "key_name": key_name,
        "key_user": truncate_key_value(key)
    })
}

/// 截断按键值到全局动作上限。
pub fn truncate_key_value(key_value: &Value) -> Value {
    match key_value {
        Value::Array(keys) => Value::Array(keys.iter().take(MAX_ACTION_KEYS).cloned().collect()),
        _ => key_value.clone(),
    }
}

fn normalize_keybind_profile(keybinds: Value) -> Value {
    let Some(mut root_object) = keybinds.as_object().cloned() else {
        return empty_keybind_profile();
    };

    let has_nested_sections = root_object.contains_key(GLOBAL_SECTION)
        || root_object.contains_key(SYSTEM_SECTION)
        || root_object.contains_key(GAME_SECTION);

    if !has_nested_sections {
        let old_game_section = Value::Object(root_object);
        return json!({
            GLOBAL_SECTION: default_global_keybinds(),
            SYSTEM_SECTION: {},
            GAME_SECTION: old_game_section
        });
    }

    let global = normalize_object_section(root_object.remove(GLOBAL_SECTION))
        .unwrap_or_else(default_global_keybinds);
    let system = normalize_object_section(root_object.remove(SYSTEM_SECTION)).unwrap_or_default();
    let game = normalize_object_section(root_object.remove(GAME_SECTION)).unwrap_or_default();

    json!({
        GLOBAL_SECTION: global,
        SYSTEM_SECTION: system,
        GAME_SECTION: game
    })
}

fn normalize_object_section(value: Option<Value>) -> Option<Map<String, Value>> {
    value.and_then(|value| value.as_object().cloned())
}

fn empty_keybind_profile() -> Value {
    json!({
        GLOBAL_SECTION: default_global_keybinds(),
        SYSTEM_SECTION: {},
        GAME_SECTION: {}
    })
}

fn default_global_keybinds() -> Map<String, Value> {
    let mut global_keybinds = Map::new();
    global_keybinds.insert(
        "screen_saver".to_string(),
        keybind_entry(&Value::String("f2".to_string()), "global.key.screen_saver"),
    );
    global_keybinds.insert(
        "boss_key".to_string(),
        keybind_entry(&Value::String("f3".to_string()), "global.key.boss_key"),
    );
    global_keybinds.insert(
        "force_stop_game".to_string(),
        keybind_entry(
            &Value::String("f4".to_string()),
            "global.key.force_stop_game",
        ),
    );
    global_keybinds
}

fn read_json_object(path: &Path) -> KeybindResult<Value> {
    let raw_json = fs::read_to_string(path)?;
    let value = serde_json::from_str::<Value>(raw_json.trim_start_matches('\u{feff}'))?;
    Ok(value)
}

fn write_json_pretty<T: serde::Serialize>(path: &Path, value: &T) -> KeybindResult<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}
