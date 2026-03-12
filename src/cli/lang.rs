use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use crate::utils::path_utils;

#[derive(Clone, Debug)]
pub struct CliLang {
    dict: HashMap<String, String>,
}

impl CliLang {
    pub fn load() -> Self {
        load_impl().unwrap_or_else(|_| fallback())
    }

    pub fn t(&self, key: &str) -> String {
        self.dict
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("[missing-cli-i18n:{key}]"))
    }

    pub fn fmt(&self, key: &str, pairs: &[(&str, &str)]) -> String {
        let mut text = self.t(key);
        for (from, to) in pairs {
            text = text.replace(from, to);
        }
        text
    }
}

fn load_impl() -> Result<CliLang> {
    let code = preferred_language_code().unwrap_or_else(|| "us-en".to_string());
    let dir = path_utils::bash_lang_dir()?;
    let preferred = dir.join(format!("{code}.json"));
    let fallback_path = dir.join("us-en.json");

    let dict = load_json_map(&preferred).or_else(|_| load_json_map(&fallback_path))?;
    Ok(CliLang { dict })
}

fn preferred_language_code() -> Option<String> {
    let path = path_utils::language_pref_file().ok()?;
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    let code = content.trim().to_ascii_lowercase();
    if code.is_empty() {
        None
    } else {
        Some(code)
    }
}

fn load_json_map(path: &Path) -> Result<HashMap<String, String>> {
    let content = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(content.trim_start_matches('\u{feff}'))?;
    let object = value
        .as_object()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid cli lang file"))?;

    let mut dict = HashMap::new();
    for (key, value) in object {
        if let Some(text) = value.as_str() {
            dict.insert(key, text.to_string());
        }
    }
    Ok(dict)
}

fn fallback() -> CliLang {
    let mut dict = HashMap::new();
    dict.insert("version.current".to_string(), "Current version: {version}".to_string());
    dict.insert("version.latest".to_string(), "Latest release: {version}".to_string());
    dict.insert(
        "version.update_available".to_string(),
        "Update available. Run 'tg -u' to update.".to_string(),
    );
    dict.insert("version.up_to_date".to_string(), "Already up to date.".to_string());
    dict.insert(
        "version.check_failed".to_string(),
        "Failed to check the latest release.".to_string(),
    );
    dict.insert(
        "updata.checking".to_string(),
        "Checking latest release...".to_string(),
    );
    dict.insert(
        "updata.no_update".to_string(),
        "Already up to date. No update required.".to_string(),
    );
    dict.insert(
        "updata.update_found".to_string(),
        "Update found: {version}".to_string(),
    );
    dict.insert(
        "updata.helper_missing".to_string(),
        "Update helper script not found.".to_string(),
    );
    dict.insert(
        "updata.asset_missing".to_string(),
        "No downloadable package found for the current platform.".to_string(),
    );
    dict.insert(
        "updata.launching".to_string(),
        "Launching update helper...".to_string(),
    );
    dict.insert(
        "remove.confirm_first".to_string(),
        "This will uninstall TUI-GAME. Continue? [y/N]".to_string(),
    );
    dict.insert(
        "remove.confirm_mode".to_string(),
        "Choose uninstall mode: [1] Keep save data  [2] Remove all data".to_string(),
    );
    dict.insert(
        "remove.confirm_second".to_string(),
        "Confirm uninstall mode '{mode}'? [y/N]".to_string(),
    );
    dict.insert(
        "remove.cancelled".to_string(),
        "Uninstall cancelled.".to_string(),
    );
    dict.insert(
        "remove.helper_missing".to_string(),
        "Remove helper script not found.".to_string(),
    );
    dict.insert(
        "remove.launching".to_string(),
        "Launching remove helper...".to_string(),
    );
    dict.insert(
        "remove.mode.keep".to_string(),
        "keep data".to_string(),
    );
    dict.insert(
        "remove.mode.full".to_string(),
        "remove all data".to_string(),
    );
    CliLang { dict }
}
