//! 独立键位绑定管理器。

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value, json};

use crate::host_engine::constant::MAX_ACTION_KEYS;
use crate::host_engine::package::package_id::PackageId;
use crate::host_engine::package::package_manager::PackageManager;
use crate::host_engine::runtime::event_dispatch::GlobalRuntimeAction;
use crate::host_engine::storage::profile_store::ProfileStore;

use super::action_schema::ActionSchema;
use super::binding::Key;
use super::default::system_defaults;
use super::resolver::{ResolvedBindings, resolve_bindings};

/// 按键对应的动作目标。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedAction {
    pub package_id: PackageId,
    pub action: String,
}

/// 键位绑定管理器。
#[derive(Clone, Debug, Default)]
pub struct KeybindManager {
    schemas: HashMap<PackageId, ActionSchema>,
    user_bindings: Value,
    disabled_packages: HashSet<PackageId>,
    resolved: ResolvedBindings,
    reverse_lookup: HashMap<Key, ResolvedAction>,
}

impl KeybindManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// 从包管理器和 profile 快照生成最终按键映射。
    pub fn resolve(
        &mut self,
        package_manager: &PackageManager,
        profile_store: &ProfileStore,
    ) -> ResolvedBindings {
        self.schemas = package_manager.action_schemas();
        self.user_bindings = profile_store.keybinds.clone();
        self.disabled_packages = package_manager.disabled_package_ids();
        self.reconcile();
        self.resolved.clone()
    }

    /// 包或用户配置变更后重新协调。
    pub fn reconcile(&mut self) {
        self.resolved =
            resolve_bindings(&self.schemas, &self.user_bindings, &self.disabled_packages);
        self.rebuild_reverse_lookup();
    }

    /// 设置用户自定义绑定。
    pub fn rebind(&mut self, package_id: &PackageId, action: &str, keys: Vec<Key>) {
        let keys = keys
            .into_iter()
            .filter(|key| !key.is_system_key())
            .take(MAX_ACTION_KEYS)
            .collect::<Vec<_>>();

        let root = ensure_object(&mut self.user_bindings);
        let game = ensure_child_object(root, "game");
        let package = ensure_child_object(game, &package_id.uid);
        package.insert(
            action.to_string(),
            json!({
                "key_user": keys_to_value(&keys)
            }),
        );
        self.reconcile();
    }

    /// O(1) 根据按键查找动作。
    pub fn action_for_key(&self, key: &Key) -> Option<&ResolvedAction> {
        self.reverse_lookup.get(key)
    }

    /// Resolve a host-global action for a physical key.
    pub fn global_action_for_key(&self, key: &Key) -> Option<GlobalRuntimeAction> {
        global_action_from_profile(&self.user_bindings, key).or_else(|| {
            system_defaults()
                .into_iter()
                .find_map(|(action, keys)| keys.contains(key).then(|| global_action(&action))?)
        })
    }

    /// 导出为兼容 `keybind.json` 的结构。
    pub fn to_profile_json(&self) -> Value {
        normalized_profile_json(&self.user_bindings)
    }

    pub fn resolved_bindings(&self) -> &ResolvedBindings {
        &self.resolved
    }

    fn rebuild_reverse_lookup(&mut self) {
        self.reverse_lookup.clear();
        for (package_id, actions) in &self.resolved {
            for (action, keys) in actions {
                for key in keys {
                    self.reverse_lookup
                        .entry(key.clone())
                        .or_insert_with(|| ResolvedAction {
                            package_id: package_id.clone(),
                            action: action.clone(),
                        });
                }
            }
        }
    }
}

fn ensure_object(value: &mut Value) -> &mut Map<String, Value> {
    if !value.is_object() {
        *value = json!({});
    }
    value.as_object_mut().expect("value was forced to object")
}

fn ensure_child_object<'a>(
    object: &'a mut Map<String, Value>,
    key: &str,
) -> &'a mut Map<String, Value> {
    let child = object.entry(key.to_string()).or_insert_with(|| json!({}));
    if !child.is_object() {
        *child = json!({});
    }
    child.as_object_mut().expect("value was forced to object")
}

fn keys_to_value(keys: &[Key]) -> Value {
    match keys {
        [single] => Value::String(single.to_string()),
        _ => Value::Array(
            keys.iter()
                .map(|key| Value::String(key.to_string()))
                .collect(),
        ),
    }
}

fn normalized_profile_json(value: &Value) -> Value {
    let mut root = value.as_object().cloned().unwrap_or_default();
    root.entry("global".to_string())
        .or_insert_with(default_global_bindings);
    root.entry("system".to_string())
        .or_insert_with(|| json!({}));
    root.entry("game".to_string()).or_insert_with(|| json!({}));
    Value::Object(root)
}

fn default_global_bindings() -> Value {
    let mut bindings = Map::new();
    for (action, keys) in system_defaults() {
        bindings.insert(
            action.clone(),
            json!({
                "key": keys_to_value(&keys),
                "key_name": format!("global.key.{action}"),
                "key_user": keys_to_value(&keys)
            }),
        );
    }
    Value::Object(bindings)
}

fn global_action_from_profile(value: &Value, key: &Key) -> Option<GlobalRuntimeAction> {
    value
        .get("global")
        .and_then(Value::as_object)?
        .iter()
        .find_map(|(action, entry)| {
            let keys = entry
                .get("key_user")
                .or_else(|| entry.get("key"))
                .and_then(profile_keys);
            keys.filter(|keys| keys.contains(key))
                .and_then(|_| global_action(action))
        })
}

fn global_action(action: &str) -> Option<GlobalRuntimeAction> {
    match action {
        "screensaver" => Some(GlobalRuntimeAction::ToggleScreensaver),
        "boss_key" => Some(GlobalRuntimeAction::ToggleBoss),
        "force_stop_game" => Some(GlobalRuntimeAction::ForceStopGame),
        _ => None,
    }
}

fn profile_keys(value: &Value) -> Option<Vec<Key>> {
    match value {
        Value::String(key) => Key::from_string(key).map(|key| vec![key]),
        Value::Array(keys) => Some(
            keys.iter()
                .filter_map(Value::as_str)
                .filter_map(Key::from_string)
                .collect(),
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::host_engine::keybind::action_schema::ActionDefault;
    use crate::host_engine::package::package_id::{PackageKind, PackageSource};

    use super::*;

    #[test]
    fn rebind_overrides_default_binding() {
        let id = PackageId::new(
            PackageSource::ThirdParty,
            PackageKind::Game,
            "mod_game_demo",
        );
        let mut manager = manager_with_schema(&id, "jump", &["space"]);

        manager.rebind(&id, "jump", vec![Key::Char('j')]);

        assert_eq!(
            manager.action_for_key(&Key::Char('j')),
            Some(&ResolvedAction {
                package_id: id.clone(),
                action: "jump".to_string()
            })
        );
        assert_eq!(manager.action_for_key(&Key::Space), None);
    }

    #[test]
    fn system_reserved_keys_are_not_package_bindable() {
        let id = PackageId::new(
            PackageSource::ThirdParty,
            PackageKind::Game,
            "mod_game_demo",
        );
        let mut manager = manager_with_schema(&id, "jump", &["f2"]);
        manager.reconcile();

        assert_eq!(manager.action_for_key(&Key::F(2)), None);
    }

    #[test]
    fn disabled_package_has_no_effective_binding() {
        let id = PackageId::new(
            PackageSource::ThirdParty,
            PackageKind::Game,
            "mod_game_demo",
        );
        let mut manager = manager_with_schema(&id, "jump", &["space"]);
        manager.disabled_packages.insert(id.clone());
        manager.reconcile();

        assert_eq!(manager.action_for_key(&Key::Space), None);
    }

    #[test]
    fn to_profile_json_keeps_nested_sections() {
        let manager = KeybindManager::new();
        let value = manager.to_profile_json();
        assert!(value.get("global").is_some());
        assert!(value.get("system").is_some());
        assert!(value.get("game").is_some());
    }

    #[test]
    fn global_action_uses_profile_binding() {
        let manager = KeybindManager {
            user_bindings: json!({
                "global": {
                    "screensaver": { "key_user": "f5" }
                }
            }),
            ..KeybindManager::new()
        };

        assert_eq!(
            manager.global_action_for_key(&Key::F(5)),
            Some(GlobalRuntimeAction::ToggleScreensaver)
        );
    }

    #[test]
    fn global_action_falls_back_to_defaults() {
        let manager = KeybindManager::new();

        assert_eq!(
            manager.global_action_for_key(&Key::F(2)),
            Some(GlobalRuntimeAction::ToggleScreensaver)
        );
    }

    fn manager_with_schema(id: &PackageId, action: &str, keys: &[&str]) -> KeybindManager {
        let mut schemas = HashMap::new();
        schemas.insert(
            id.clone(),
            ActionSchema {
                actions: HashMap::from([(
                    action.to_string(),
                    ActionDefault {
                        default_keys: keys.iter().map(|key| key.to_string()).collect(),
                        display_name: action.to_string(),
                    },
                )]),
            },
        );
        let mut manager = KeybindManager {
            schemas,
            user_bindings: json!({}),
            disabled_packages: HashSet::new(),
            resolved: HashMap::new(),
            reverse_lookup: HashMap::new(),
        };
        manager.reconcile();
        manager
    }
}
