//! 宿主 UI 动作映射

use std::collections::HashMap;

use serde_json::Value;

use crate::host_engine::boot::preload::persistent_data::keybind_profile;
use crate::host_engine::constant::MAX_ACTION_KEYS;
use crate::host_engine::runtime::ui_page::action_defaults;

/// 单个 UI 动作定义。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiActionBinding {
    pub action: String,
    pub keys: Vec<String>,
}

/// UI 页面动作映射。
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UiActionMap {
    key_to_action: HashMap<String, String>,
    actions: Value,
}

impl UiActionMap {
    /// 从宿主内置 actions 中读取指定页面动作。
    pub fn from_page(page_name: &str, keybinds: &Value) -> Self {
        let mut action_map = Self::default();
        let Some(page_actions) =
            action_defaults::page_actions(page_name).and_then(Value::as_object)
        else {
            return action_map;
        };

        action_map.actions =
            Value::Object(normalize_page_actions(page_actions, page_name, keybinds));
        for (action_name, action_value) in page_actions {
            let stored_key = stored_system_key(keybinds, page_name, action_name);
            let key_value = stored_key
                .filter(|value| parse_keys(Some(value)).is_some_and(|keys| !keys.is_empty()))
                .or_else(|| action_value.get("key"));
            if let Some(keys) = parse_keys(key_value) {
                for key in keys {
                    action_map
                        .key_to_action
                        .insert(normalize_key(key.as_str()), action_name.clone());
                }
            }
        }

        action_map
    }

    /// Compatibility wrapper for older state code. The manifest argument is ignored because
    /// official host UI actions are now hardcoded in Rust.
    pub fn from_manifest_page(_manifest: &Value, page_name: &str, keybinds: &Value) -> Self {
        Self::from_page(page_name, keybinds)
    }

    /// 查找物理键对应动作。
    pub fn action_for_key(&self, key: &str) -> Option<String> {
        self.key_to_action.get(&normalize_key(key)).cloned()
    }

    /// 返回当前页面 actions JSON。
    pub fn actions_value(&self) -> Value {
        self.actions.clone()
    }
}

fn parse_keys(value: Option<&Value>) -> Option<Vec<String>> {
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

fn normalize_page_actions(
    page_actions: &serde_json::Map<String, Value>,
    page_name: &str,
    keybinds: &Value,
) -> serde_json::Map<String, Value> {
    let mut normalized_actions = serde_json::Map::new();
    for (action_name, action_value) in page_actions {
        let mut normalized_action = action_value.clone();
        let stored_key = stored_system_key(keybinds, page_name, action_name)
            .filter(|value| parse_keys(Some(value)).is_some_and(|keys| !keys.is_empty()))
            .cloned();
        if let Some(action_object) = normalized_action.as_object_mut() {
            if let Some(stored_key) = stored_key {
                action_object.insert("key".to_string(), truncate_key_value(&stored_key));
            } else if let Some(key_value) = action_object.get_mut("key") {
                *key_value = truncate_key_value(key_value);
            }
        }
        normalized_actions.insert(action_name.clone(), normalized_action);
    }
    normalized_actions
}

fn stored_system_key<'a>(
    keybinds: &'a Value,
    page_name: &str,
    action_name: &str,
) -> Option<&'a Value> {
    keybind_profile::system_section(keybinds)
        .and_then(|system| system.get(page_name))
        .and_then(Value::as_object)
        .and_then(|page| page.get(action_name))
        .and_then(|action| action.get("key_user"))
}

fn truncate_key_value(key_value: &Value) -> Value {
    match key_value {
        Value::Array(keys) => Value::Array(keys.iter().take(MAX_ACTION_KEYS).cloned().collect()),
        _ => key_value.clone(),
    }
}

fn normalize_key(key: &str) -> String {
    key.to_ascii_lowercase()
}
