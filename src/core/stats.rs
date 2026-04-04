use std::collections::{HashMap, HashSet};
use std::fs;

use anyhow::Result;
use serde_json::{Map, Value as JsonValue};

use crate::app::stats::{self, GameStats};
use crate::utils::path_utils;

/// 统一运行时读取宿主统计信息。
pub fn load_all() -> HashMap<String, GameStats> {
    stats::load_stats()
}

pub fn runtime_stats_path() -> Result<std::path::PathBuf> {
    Ok(path_utils::app_data_dir()?.join("runtime_best_scores.json"))
}

pub fn read_runtime_best_score(game_id: &str) -> Option<JsonValue> {
    let path = runtime_stats_path().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    let store =
        serde_json::from_str::<Map<String, JsonValue>>(raw.trim_start_matches('\u{feff}')).ok()?;
    store.get(game_id).cloned()
}

pub fn write_runtime_best_score(game_id: &str, value: &JsonValue) -> Result<()> {
    let path = runtime_stats_path()?;
    path_utils::ensure_parent_dir(&path)?;
    let mut store = if path.exists() {
        let raw = fs::read_to_string(&path)?;
        serde_json::from_str::<Map<String, JsonValue>>(raw.trim_start_matches('\u{feff}'))
            .unwrap_or_default()
    } else {
        Map::new()
    };
    store.insert(game_id.to_string(), value.clone());
    fs::write(path, serde_json::to_string_pretty(&store)?)?;
    Ok(())
}

pub fn prune_runtime_scores(valid_game_ids: impl IntoIterator<Item = String>) -> Result<()> {
    let path = runtime_stats_path()?;
    if !path.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(&path)?;
    let mut store =
        serde_json::from_str::<Map<String, JsonValue>>(raw.trim_start_matches('\u{feff}'))
            .unwrap_or_default();
    let valid: HashSet<String> = valid_game_ids.into_iter().collect();
    store.retain(|key, _| valid.contains(key));
    fs::write(path, serde_json::to_string_pretty(&store)?)?;
    Ok(())
}
