use std::collections::{HashMap, HashSet};
use std::fs;

use anyhow::Result;
use serde_json::{Map, Value as JsonValue};

use crate::utils::path_utils;

fn merge_objects(base: &mut Map<String, JsonValue>, overlay: &Map<String, JsonValue>) {
    for (key, value) in overlay {
        match (base.get_mut(key), value) {
            (Some(JsonValue::Object(existing)), JsonValue::Object(incoming)) => {
                merge_objects(existing, incoming);
            }
            _ => {
                base.insert(key.clone(), value.clone());
            }
        }
    }
}

fn read_store() -> Result<Map<String, JsonValue>> {
    let path = path_utils::best_scores_file()?;
    if !path.exists() {
        return Ok(Map::new());
    }
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str::<Map<String, JsonValue>>(raw.trim_start_matches('\u{feff}'))
        .unwrap_or_default())
}

fn write_store(store: &Map<String, JsonValue>) -> Result<()> {
    let path = path_utils::best_scores_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, serde_json::to_string_pretty(store)?)?;
    Ok(())
}

pub fn read_runtime_best_score(game_id: &str) -> Option<JsonValue> {
    read_store().ok()?.get(game_id).cloned()
}

pub fn write_runtime_best_score(game_id: &str, value: &JsonValue) -> Result<()> {
    let mut store = read_store()?;
    let merged = match (store.remove(game_id), value) {
        (Some(JsonValue::Object(mut existing)), JsonValue::Object(incoming)) => {
            merge_objects(&mut existing, incoming);
            JsonValue::Object(existing)
        }
        _ => value.clone(),
    };
    store.insert(game_id.to_string(), merged);
    write_store(&store)
}

pub fn prune_runtime_scores(valid_game_ids: impl IntoIterator<Item = String>) -> Result<()> {
    let valid: HashSet<String> = valid_game_ids.into_iter().collect();
    let mut store = read_store()?;
    store.retain(|key, _| valid.contains(key));
    write_store(&store)
}

pub fn load_all() -> HashMap<String, JsonValue> {
    read_store()
        .map(|store| store.into_iter().collect())
        .unwrap_or_default()
}
