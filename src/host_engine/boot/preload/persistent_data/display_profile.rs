//! 显示设置持久化配置

use crate::host_engine::boot::environment::data_dirs;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const DISPLAY_STATE_PATH: &str = "data/profiles/display_state.json";
const DEFAULT_IDLE_THRESHOLD_SECS: u64 = 60;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DisplayProfile {
    pub mod_badge: bool,
    pub theme: String,
    pub idle_threshold: u64,
    pub idle_enter_screensaver: bool,
    pub host_status: bool,
    pub screensaver_mode: String,
    pub boss_mode: String,
    pub screensaver_list: DisplayOverlayProfile,
    pub boss_list: DisplayOverlayProfile,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DisplayOverlayProfile {
    pub order: Vec<String>,
    pub enabled: BTreeMap<String, bool>,
    pub cursor: usize,
}

impl Default for DisplayProfile {
    fn default() -> Self {
        Self {
            mod_badge: true,
            theme: "system".to_string(),
            idle_threshold: DEFAULT_IDLE_THRESHOLD_SECS,
            idle_enter_screensaver: false,
            host_status: false,
            screensaver_mode: "ordered".to_string(),
            boss_mode: "ordered".to_string(),
            screensaver_list: DisplayOverlayProfile::default(),
            boss_list: DisplayOverlayProfile::default(),
        }
    }
}

impl DisplayProfile {
    pub fn from_value(value: &Value) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }

    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or_else(|_| json!({}))
    }

    pub fn persist_default_path(&self) -> Result<(), Box<dyn std::error::Error>> {
        persist_display_profile(&data_dirs::root_dir().join(DISPLAY_STATE_PATH), self)
    }

    pub fn normalize(&mut self) {
        if self.theme.trim().is_empty() {
            self.theme = "system".to_string();
        }
        if !matches!(self.screensaver_mode.as_str(), "ordered" | "random" | "off") {
            self.screensaver_mode = "ordered".to_string();
        }
        if !matches!(self.boss_mode.as_str(), "ordered" | "random" | "off") {
            self.boss_mode = "ordered".to_string();
        }
        if !matches!(self.idle_threshold, 0 | 30 | 60 | 300 | 600) {
            self.idle_threshold = DEFAULT_IDLE_THRESHOLD_SECS;
        }
    }
}

pub fn load_display_profile(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    if !path.is_file() {
        let profile = DisplayProfile::default();
        persist_display_profile(path, &profile)?;
        return Ok(profile.to_value());
    }
    let raw_json = fs::read_to_string(path)?;
    let value = serde_json::from_str::<Value>(raw_json.trim_start_matches('\u{feff}'))
        .unwrap_or_else(|_| DisplayProfile::default().to_value());
    let mut profile = DisplayProfile::from_value(&value);
    profile.normalize();
    persist_display_profile(path, &profile)?;
    Ok(profile.to_value())
}

pub fn persist_display_profile(
    path: &Path,
    profile: &DisplayProfile,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(path, serde_json::to_string_pretty(&profile.to_value())?)?;
    Ok(())
}
