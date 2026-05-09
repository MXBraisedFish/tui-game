//! UI 包动作映射

use std::collections::HashMap;

use serde_json::Value;

use crate::host_engine::constant::MAX_ACTION_KEYS;

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
    /// 从 official_ui package.json 中读取指定页面的 actions。
    pub fn from_manifest_page(manifest: &Value, page_name: &str) -> Self {
        let mut action_map = Self::default();
        let Some(page_actions) = manifest
            .get("actions")
            .and_then(Value::as_object)
            .and_then(|actions| actions.get(page_name))
            .and_then(Value::as_object)
        else {
            return action_map;
        };

        action_map.actions = Value::Object(normalize_page_actions(page_actions));
        for (action_name, action_value) in page_actions {
            if let Some(keys) = parse_keys(action_value.get("key")) {
                for key in keys {
                    action_map
                        .key_to_action
                        .insert(normalize_key(key.as_str()), action_name.clone());
                }
            }
        }

        action_map
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
) -> serde_json::Map<String, Value> {
    let mut normalized_actions = serde_json::Map::new();
    for (action_name, action_value) in page_actions {
        let mut normalized_action = action_value.clone();
        if let Some(action_object) = normalized_action.as_object_mut() {
            if let Some(key_value) = action_object.get_mut("key") {
                *key_value = truncate_key_value(key_value);
            }
        }
        normalized_actions.insert(action_name.clone(), normalized_action);
    }
    normalized_actions
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
