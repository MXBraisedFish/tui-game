use anyhow::Result;
use serde_json::{Map, Value as JsonValue};
use std::fs;
use std::collections::HashMap;

use crate::utils::path_utils;

fn empty_store() -> Map<String, JsonValue> {
    let mut store = Map::new();
    store.insert("continue".to_string(), JsonValue::Object(Map::new()));
    store.insert("data".to_string(), JsonValue::Object(Map::new()));
    store
}

fn read_store() -> Result<Map<String, JsonValue>> {
    let path = path_utils::saves_file()?;
    if !path.exists() {
        return Ok(empty_store());
    }
    let raw = fs::read_to_string(path)?;
    let mut store =
        serde_json::from_str::<Map<String, JsonValue>>(raw.trim_start_matches('\u{feff}'))
            .unwrap_or_else(|_| empty_store());
    if !matches!(store.get("continue"), Some(JsonValue::Object(_))) {
        store.insert("continue".to_string(), JsonValue::Object(Map::new()));
    }
    if !matches!(store.get("data"), Some(JsonValue::Object(_))) {
        store.insert("data".to_string(), JsonValue::Object(Map::new()));
    }
    Ok(store)
}

fn write_store(store: &Map<String, JsonValue>) -> Result<()> {
    let path = path_utils::saves_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, serde_json::to_string_pretty(store)?)?;
    Ok(())
}

const KEYBINDINGS_SLOT: &str = "__keybindings";

pub fn sanitize_runtime_save_stem(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "runtime_save".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn save_data_slot(game_id: &str, slot: &str, value: &JsonValue) -> Result<()> {
    let mut store = read_store()?;
    let data = store
        .get_mut("data")
        .and_then(JsonValue::as_object_mut)
        .expect("data object");
    let game_slots = data
        .entry(game_id.to_string())
        .or_insert_with(|| JsonValue::Object(Map::new()))
        .as_object_mut()
        .expect("game data object");
    game_slots.insert(slot.to_string(), value.clone());
    write_store(&store)
}

pub fn load_data_slot(game_id: &str, slot: &str) -> Result<Option<JsonValue>> {
    let store = read_store()?;
    Ok(store
        .get("data")
        .and_then(JsonValue::as_object)
        .and_then(|data| data.get(game_id))
        .and_then(JsonValue::as_object)
        .and_then(|slots| slots.get(slot))
        .cloned())
}

pub fn save_continue(game_id: &str, value: &JsonValue) -> Result<()> {
    let mut store = read_store()?;
    let continue_map = store
        .get_mut("continue")
        .and_then(JsonValue::as_object_mut)
        .expect("continue object");
    continue_map.clear();
    continue_map.insert(game_id.to_string(), value.clone());
    write_store(&store)
}

pub fn load_continue(game_id: &str) -> Result<Option<JsonValue>> {
    let store = read_store()?;
    Ok(store
        .get("continue")
        .and_then(JsonValue::as_object)
        .and_then(|continue_map| continue_map.get(game_id))
        .cloned())
}

pub fn latest_saved_game_id() -> Option<String> {
    let store = read_store().ok()?;
    store
        .get("continue")
        .and_then(JsonValue::as_object)
        .and_then(|continue_map| continue_map.keys().next().cloned())
}

pub fn clear_active_game_save() -> Result<()> {
    let mut store = read_store()?;
    if let Some(continue_map) = store.get_mut("continue").and_then(JsonValue::as_object_mut) {
        continue_map.clear();
    }
    write_store(&store)
}

pub fn game_has_continue_save(game_id: &str) -> bool {
    load_continue(game_id).ok().flatten().is_some()
}

pub fn save_keybindings(game_id: &str, bindings: &HashMap<String, Vec<String>>) -> Result<()> {
    let value = serde_json::to_value(bindings)?;
    save_data_slot(game_id, KEYBINDINGS_SLOT, &value)
}

pub fn load_keybindings(game_id: &str) -> Result<HashMap<String, Vec<String>>> {
    let Some(value) = load_data_slot(game_id, KEYBINDINGS_SLOT)? else {
        return Ok(HashMap::new());
    };
    Ok(serde_json::from_value(value).unwrap_or_default())
}
