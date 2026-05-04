//! UI 包动作映射

use std::collections::HashMap;

use serde_json::Value;

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

        action_map.actions = Value::Object(page_actions.clone());
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
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect(),
        ),
        _ => None,
    }
}

fn normalize_key(key: &str) -> String {
    key.to_ascii_lowercase()
}
