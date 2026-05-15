//! 安全默认设置持久化。

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

#[derive(Clone, Debug)]
pub struct SecurityProfile {
    pub default_safe_mode: bool,
    pub default_mod_game_enabled: bool,
    pub default_mod_saver_enabled: bool,
    pub default_mod_boss_enabled: bool,
}

impl Default for SecurityProfile {
    fn default() -> Self {
        Self {
            default_safe_mode: true,
            default_mod_game_enabled: true,
            default_mod_saver_enabled: true,
            default_mod_boss_enabled: true,
        }
    }
}

impl SecurityProfile {
    pub fn from_value(value: &Value) -> Self {
        Self {
            default_safe_mode: value
                .get("default_safe_mode")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            default_mod_game_enabled: value
                .get("default_mod_game_enabled")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            default_mod_saver_enabled: value
                .get("default_mod_saver_enabled")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            default_mod_boss_enabled: value
                .get("default_mod_boss_enabled")
                .and_then(Value::as_bool)
                .unwrap_or(true),
        }
    }

    pub fn to_value(&self) -> Value {
        json!({
            "default_safe_mode": self.default_safe_mode,
            "default_mod_game_enabled": self.default_mod_game_enabled,
            "default_mod_saver_enabled": self.default_mod_saver_enabled,
            "default_mod_boss_enabled": self.default_mod_boss_enabled,
        })
    }
}

pub fn load_from_default_path() -> SecurityProfile {
    read_profile(&profile_path()).unwrap_or_default()
}

pub fn persist_to_default_path(profile: &SecurityProfile) -> std::io::Result<()> {
    write_profile(&profile_path(), profile)
}

fn read_profile(path: &Path) -> Option<SecurityProfile> {
    let raw_json = fs::read_to_string(path).ok()?;
    let value = serde_json::from_str::<Value>(raw_json.trim_start_matches('\u{feff}')).ok()?;
    Some(SecurityProfile::from_value(&value))
}

fn write_profile(path: &Path, profile: &SecurityProfile) -> std::io::Result<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(path, serde_json::to_string_pretty(&profile.to_value())?)
}

fn profile_path() -> PathBuf {
    root_dir().join("data/profiles/security_state.json")
}

fn root_dir() -> PathBuf {
    std::env::current_dir()
        .ok()
        .filter(|path| path.join("assets").exists() || path.join("Cargo.toml").exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(Path::to_path_buf))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}
