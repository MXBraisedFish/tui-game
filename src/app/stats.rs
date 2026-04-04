use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::utils::path_utils;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct GameStats {
    pub high_score: u32,
    pub max_duration_sec: u64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct StatsFile {
    #[serde(default)]
    games: HashMap<String, GameStats>,
}

pub fn load_stats() -> HashMap<String, GameStats> {
    match load_stats_inner() {
        Ok(map) => map,
        Err(_) => HashMap::new(),
    }
}

pub fn update_game_stats(game_id: &str, score: u32, duration_sec: u64) -> Result<()> {
    let path = stats_file_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, "{\n  \"games\": {}\n}\n")?;
    }

    let content = fs::read_to_string(&path)?;
    let mut parsed: StatsFile = serde_json::from_str(&content).unwrap_or_default();
    let entry = parsed.games.entry(game_id.to_string()).or_default();
    entry.high_score = entry.high_score.max(score);
    entry.max_duration_sec = entry.max_duration_sec.max(duration_sec);

    let payload = serde_json::to_string_pretty(&parsed)?;
    fs::write(path, payload)?;
    Ok(())
}

fn load_stats_inner() -> Result<HashMap<String, GameStats>> {
    let path = stats_file_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, "{\n  \"games\": {}\n}\n")?;
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(path)?;
    let parsed: StatsFile = serde_json::from_str(&content).unwrap_or_default();
    Ok(parsed.games)
}

pub fn format_duration(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

fn stats_file_path() -> PathBuf {
    match path_utils::stats_file() {
        Ok(path) => path,
        Err(_) => PathBuf::from("stats.json"),
    }
}
