//! 游戏动作按键映射

use serde_json::Value;

use crate::host_engine::boot::preload::game_modules::GameModule;
use crate::host_engine::boot::preload::persistent_data::keybind_profile;
use crate::host_engine::constant::MAX_ACTION_KEYS;

/// 查找物理键对应的游戏动作。
pub fn action_for_key(game_module: &GameModule, keybinds: &Value, key: &str) -> Option<String> {
    let normalized_key = normalize_key(key, game_module.game.case_sensitive);

    for action_name in game_module.game.actions.keys() {
        for action_key in action_keys(game_module, keybinds, action_name) {
            if normalize_key(action_key.as_str(), game_module.game.case_sensitive) == normalized_key
            {
                return Some(action_name.clone());
            }
        }
    }

    None
}

fn action_keys(game_module: &GameModule, keybinds: &Value, action_name: &str) -> Vec<String> {
    let user_key = keybinds
        .get(keybind_profile::GAME_SECTION)
        .and_then(|game_keybinds| game_keybinds.get(game_module.uid.as_str()))
        .and_then(|game_keybinds| game_keybinds.get(action_name))
        .and_then(|action_keybind| action_keybind.get("key_user"));

    if let Some(keys) = parse_key_value(user_key) {
        return keys;
    }

    game_module
        .game
        .actions
        .get(action_name)
        .and_then(|action_binding| parse_key_value(Some(&action_binding.key)))
        .unwrap_or_default()
}

fn parse_key_value(value: Option<&Value>) -> Option<Vec<String>> {
    match value? {
        Value::String(key) => Some(vec![key.clone()]),
        Value::Array(keys) => Some(
            keys.iter()
                .take(MAX_ACTION_KEYS)
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect(),
        ),
        _ => None,
    }
}

fn normalize_key(key: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        key.to_string()
    } else {
        key.to_ascii_lowercase()
    }
}
