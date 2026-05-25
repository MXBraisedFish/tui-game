//! 包动作键位解析。

use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::host_engine::boot::preload::persistent_data::keybind_profile;
use crate::host_engine::constant::MAX_ACTION_KEYS;
use crate::host_engine::package::package_id::PackageId;

use super::action_schema::ActionSchema;
use super::binding::Key;

pub type ResolvedBindings = HashMap<PackageId, HashMap<String, Vec<Key>>>;

/// 将包声明和用户配置解析为最终生效绑定。
pub fn resolve_bindings(
    schemas: &HashMap<PackageId, ActionSchema>,
    user_bindings: &Value,
    disabled_packages: &HashSet<PackageId>,
) -> ResolvedBindings {
    let mut resolved = HashMap::new();

    for (package_id, schema) in schemas {
        if disabled_packages.contains(package_id) {
            continue;
        }

        let mut actions = HashMap::new();
        for (action, default) in &schema.actions {
            let keys = user_keys(user_bindings, &package_id.uid, action)
                .unwrap_or_else(|| parse_keys(&default.default_keys));
            let filtered = keys
                .into_iter()
                .filter(|key| !key.is_system_key())
                .take(MAX_ACTION_KEYS)
                .collect::<Vec<_>>();
            actions.insert(action.clone(), filtered);
        }
        resolved.insert(package_id.clone(), actions);
    }

    resolved
}

fn user_keys(user_bindings: &Value, uid: &str, action: &str) -> Option<Vec<Key>> {
    let value = user_bindings
        .get(keybind_profile::GAME_SECTION)
        .and_then(|game_bindings| game_bindings.get(uid))
        .and_then(|package_bindings| package_bindings.get(action))
        .and_then(|binding| binding.get("key_user"))?;
    parse_key_value(value)
}

fn parse_key_value(value: &Value) -> Option<Vec<Key>> {
    match value {
        Value::String(key) => Key::from_string(key).map(|key| vec![key]),
        Value::Array(keys) => Some(
            keys.iter()
                .take(MAX_ACTION_KEYS)
                .filter_map(Value::as_str)
                .filter_map(Key::from_string)
                .collect(),
        ),
        _ => None,
    }
}

fn parse_keys(keys: &[String]) -> Vec<Key> {
    keys.iter()
        .filter_map(|key| Key::from_string(key))
        .collect()
}
