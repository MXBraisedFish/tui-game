//! 官方 UI 按键默认值持久化。

use crate::host_engine::boot::environment::data_dirs;

use serde_json::Value;

use crate::host_engine::boot::preload::persistent_data::keybind_profile;

use super::manifest::OfficialUiRegistry;

type CacheResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 将官方 UI 默认按键信息写入 system 分区。已有用户数据不会被覆盖。
pub fn persist_default_system_keybinds(registry: &OfficialUiRegistry) -> CacheResult<()> {
    let path = data_dirs::root_dir().join("data/profiles/keybind.json");
    let mut root = keybind_profile::read_keybind_profile(&path);
    let system_section = root
        .as_object_mut()
        .and_then(|root_object| root_object.get_mut(keybind_profile::SYSTEM_SECTION))
        .and_then(Value::as_object_mut)
        .expect("normalized keybind profile must contain system object");

    for package in &registry.packages {
        let Some(actions) = package.manifest.get("actions").and_then(Value::as_object) else {
            continue;
        };

        for (page_name, page_actions) in actions {
            let Some(page_actions) = page_actions.as_object() else {
                continue;
            };
            let page_entry = system_section
                .entry(page_name.clone())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            let Some(page_object) = page_entry.as_object_mut() else {
                continue;
            };

            for (action_name, action_value) in page_actions {
                let Some(key_value) = action_value.get("key") else {
                    continue;
                };
                let key_name = action_value
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default();

                page_object
                    .entry(action_name.clone())
                    .or_insert_with(|| keybind_profile::keybind_entry(key_value, key_name));
            }
        }
    }

    keybind_profile::write_keybind_profile(&path, &root)
}
