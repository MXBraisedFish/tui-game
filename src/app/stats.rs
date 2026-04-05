use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct GameStats {
    pub high_score: u32,
    pub max_duration_sec: u64,
}

pub fn load_stats() -> HashMap<String, GameStats> {
    HashMap::new()
}

pub fn update_game_stats(_game_id: &str, _score: u32, _duration_sec: u64) -> Result<()> {
    Ok(())
}

pub fn format_duration(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    format!("{h:02}:{m:02}:{s:02}")
}
