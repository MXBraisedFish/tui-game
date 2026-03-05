use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

use crate::utils::path_utils;

const GITHUB_API_LATEST: &str = "https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest";
const FALLBACK_RELEASE_URL: &str = "https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest";
pub const GITHUB_TOKEN: &str = "";
pub const CURRENT_VERSION_TAG: &str = "0.10.2";

#[derive(Clone, Debug)]
pub struct UpdateNotification {
    pub latest_version: String,
    pub release_url: String,
}

#[derive(Clone, Debug)]
pub enum UpdaterEvent {
    LatestVersion(UpdateNotification),
    NewVersion(UpdateNotification),
    NoUpdate,
}

#[derive(Debug)]
pub struct Updater {
    receiver: Receiver<UpdaterEvent>,
}

#[derive(Clone, Debug, Deserialize)]
struct ReleaseResponse {
    tag_name: String,
    html_url: Option<String>,
}

impl Updater {
    /// Starts a background updater check thread.
    pub fn spawn(current_version: &str) -> Self {
        let (tx, rx) = mpsc::channel();
        let current = normalize_tag(current_version);
        let _ = write_current_version_cache(&current);

        thread::spawn(move || {
            if let Ok(result) = fetch_latest_release() {
                if let Some(latest) = result {
                    let _ = tx.send(UpdaterEvent::LatestVersion(latest.clone()));
                    if is_version_newer(&latest.latest_version, &current) {
                        let _ = tx.send(UpdaterEvent::NewVersion(latest));
                    } else {
                        let _ = tx.send(UpdaterEvent::NoUpdate);
                    }
                }
            }
        });

        Self { receiver: rx }
    }

    /// Non-blocking poll for updater events.
    pub fn try_recv(&self) -> Option<UpdaterEvent> {
        self.receiver.try_recv().ok()
    }
}

fn fetch_latest_release() -> Result<Option<UpdateNotification>> {
    let client = Client::builder().timeout(Duration::from_secs(8)).build()?;
    let mut req = client
        .get(GITHUB_API_LATEST)
        .header(USER_AGENT, "tui-game-updater")
        .header(ACCEPT, "application/vnd.github+json");

    if !GITHUB_TOKEN.is_empty() {
        req = req.header(AUTHORIZATION, format!("Bearer {}", GITHUB_TOKEN));
    }

    let response = match req.send() {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    if !response.status().is_success() {
        return Ok(None);
    }

    let payload: ReleaseResponse = match response.json() {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };

    let latest_tag = normalize_tag(&payload.tag_name);
    let release_url = payload
        .html_url
        .unwrap_or_else(|| FALLBACK_RELEASE_URL.to_string());

    Ok(Some(UpdateNotification {
        latest_version: latest_tag,
        release_url,
    }))
}

fn write_current_version_cache(current_version: &str) -> Result<()> {
    let path = path_utils::updater_cache_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, format!("\"{}\"\n", normalize_tag(current_version)))?;
    Ok(())
}

fn normalize_tag(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return format!("v{}", CURRENT_VERSION_TAG.trim_start_matches(['v', 'V']));
    }
    if trimmed.starts_with('v') || trimmed.starts_with('V') {
        format!("v{}", trimmed[1..].trim())
    } else {
        format!("v{}", trimmed)
    }
}


fn parse_version_segments(version: &str) -> Option<Vec<u64>> {
    let clean = version.trim().trim_start_matches(['v', 'V']);
    if clean.is_empty() {
        return None;
    }

    let mut out = Vec::new();
    for part in clean.split('.') {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let Ok(num) = part.parse::<u64>() else {
            return None;
        };
        out.push(num);
    }

    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn compare_versions(remote: &str, current: &str) -> Option<Ordering> {
    let a = parse_version_segments(remote)?;
    let b = parse_version_segments(current)?;
    let max_len = a.len().max(b.len());

    for i in 0..max_len {
        let av = *a.get(i).unwrap_or(&0);
        let bv = *b.get(i).unwrap_or(&0);
        match av.cmp(&bv) {
            Ordering::Equal => {}
            non_eq => return Some(non_eq),
        }
    }

    Some(Ordering::Equal)
}

fn is_version_newer(remote: &str, current: &str) -> bool {
    matches!(compare_versions(remote, current), Some(Ordering::Greater))
}

/// Runs external updater script (version.bat/version.sh) and returns whether it was started.
pub fn run_external_update_script(notification: &UpdateNotification) -> Result<bool> {
    let runtime = path_utils::runtime_dir()?;
    let bat = runtime.join("version.bat");
    let sh = runtime.join("version.sh");

    let Some(script) = select_version_script(&bat, &sh) else {
        return Ok(false);
    };

    let ext = script
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if ext == "bat" {
        let _child = Command::new("cmd")
            .arg("/C")
            .arg(script.as_os_str())
            .arg(notification.latest_version.as_str())
            .arg(notification.release_url.as_str())
            .spawn()?;
        return Ok(true);
    }

    let _child = Command::new("sh")
        .arg(script.as_os_str())
        .arg(notification.latest_version.as_str())
        .arg(notification.release_url.as_str())
        .spawn()?;
    Ok(true)
}

fn select_version_script(bat: &Path, sh: &Path) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if bat.exists() {
            return Some(bat.to_path_buf());
        }
        if sh.exists() {
            return Some(sh.to_path_buf());
        }
        return None;
    }
    #[cfg(not(target_os = "windows"))]
    {
        if sh.exists() {
            return Some(sh.to_path_buf());
        }
        if bat.exists() {
            return Some(bat.to_path_buf());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{compare_versions, is_version_newer, normalize_tag, select_version_script};

    #[test]
    fn normalize_tag_adds_prefix() {
        assert_eq!(normalize_tag("0.1.4"), "v0.1.4");
        assert_eq!(normalize_tag("v0.1.4"), "v0.1.4");
    }

    #[test]
    fn script_selection_prefers_bat_then_sh() {
        let base = std::env::temp_dir().join("tui_game_updater_script_select");
        let _ = std::fs::create_dir_all(&base);
        let bat = base.join("version.bat");
        let sh = base.join("version.sh");

        let _ = std::fs::remove_file(&bat);
        let _ = std::fs::remove_file(&sh);
        assert!(select_version_script(&bat, &sh).is_none());

        let _ = std::fs::write(&sh, "echo sh");
        assert_eq!(select_version_script(&bat, &sh), Some(sh.clone()));

        let _ = std::fs::write(&bat, "echo bat");
        #[cfg(target_os = "windows")]
        assert_eq!(select_version_script(&bat, &sh), Some(bat.clone()));
        #[cfg(not(target_os = "windows"))]
        assert_eq!(select_version_script(&bat, &sh), Some(sh.clone()));

        let _ = std::fs::remove_file(&bat);
        let _ = std::fs::remove_file(&sh);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn version_compare_is_semantic() {
        use std::cmp::Ordering;

        assert_eq!(compare_versions("v0.3.2", "v0.3.1"), Some(Ordering::Greater));
        assert_eq!(compare_versions("v0.3.1", "v0.3.2"), Some(Ordering::Less));
        assert_eq!(compare_versions("v0.3.2", "v0.3.2"), Some(Ordering::Equal));
        assert_eq!(compare_versions("v0.3.10", "v0.3.2"), Some(Ordering::Greater));

        assert!(is_version_newer("v1.0.0", "v0.9.9"));
        assert!(!is_version_newer("v0.9.9", "v1.0.0"));
        assert!(!is_version_newer("v1.0.0", "v1.0.0"));
    }
}
