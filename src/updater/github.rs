use std::cmp::Ordering;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

use crate::utils::path_utils;

const GITHUB_API_LATEST: &str =
    "https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest";
const FALLBACK_RELEASE_URL: &str = "https://github.com/MXBraisedFish/TUI-GAME/releases/latest";
pub const GITHUB_TOKEN: &str = "";
pub const CURRENT_VERSION_TAG: &str = "0.10.6";

#[derive(Clone, Debug)]
pub struct UpdateNotification {
    pub latest_version: String,
    pub release_url: String,
}

#[derive(Clone, Debug)]
pub struct ReleaseDownload {
    pub latest_version: String,
    pub release_url: String,
    pub asset_name: String,
    pub asset_url: String,
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
    #[serde(default)]
    assets: Vec<ReleaseAsset>,
}

#[derive(Clone, Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

impl Updater {
    pub fn spawn(current_version: &str) -> Self {
        let (tx, rx) = mpsc::channel();
        let current = normalize_tag(current_version);
        let _ = write_current_version_cache(&current);

        thread::spawn(move || {
            if let Ok(Some(latest)) = fetch_latest_release() {
                let _ = tx.send(UpdaterEvent::LatestVersion(latest.clone()));
                if is_version_newer(&latest.latest_version, &current) {
                    let _ = tx.send(UpdaterEvent::NewVersion(latest));
                } else {
                    let _ = tx.send(UpdaterEvent::NoUpdate);
                }
            }
        });

        Self { receiver: rx }
    }

    pub fn try_recv(&self) -> Option<UpdaterEvent> {
        self.receiver.try_recv().ok()
    }
}

pub fn latest_release_notification() -> Result<Option<UpdateNotification>> {
    fetch_latest_release()
}

pub fn latest_release_download() -> Result<Option<ReleaseDownload>> {
    let payload = match fetch_latest_release_payload()? {
        Some(payload) => payload,
        None => return Ok(None),
    };

    let latest_version = normalize_tag(&payload.tag_name);
    let release_url = payload
        .html_url
        .clone()
        .unwrap_or_else(|| FALLBACK_RELEASE_URL.to_string());
    let asset_name = platform_asset_name().to_string();

    let asset = payload.assets.into_iter().find(|asset| {
        asset
            .name
            .eq_ignore_ascii_case(platform_asset_name())
    });

    let Some(asset) = asset else {
        return Ok(None);
    };

    Ok(Some(ReleaseDownload {
        latest_version,
        release_url,
        asset_name,
        asset_url: asset.browser_download_url,
    }))
}

pub fn normalize_tag(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return format!("v{}", CURRENT_VERSION_TAG.trim_start_matches(['v', 'V']));
    }
    if trimmed.starts_with('v') || trimmed.starts_with('V') {
        format!("v{}", trimmed[1..].trim())
    } else {
        format!("v{trimmed}")
    }
}

pub fn is_version_newer(remote: &str, current: &str) -> bool {
    matches!(compare_versions(remote, current), Some(Ordering::Greater))
}

pub fn platform_asset_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "tui-game-windows.zip"
    }
    #[cfg(target_os = "linux")]
    {
        "tui-game-linux.tar.gz"
    }
    #[cfg(target_os = "macos")]
    {
        "tui-game-macos.zip"
    }
}

pub fn write_current_version_cache(current_version: &str) -> Result<()> {
    let path = path_utils::updater_cache_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, format!("\"{}\"\n", normalize_tag(current_version)))?;
    Ok(())
}

pub fn run_external_update_script(notification: &UpdateNotification) -> Result<bool> {
    let updata_bin = path_utils::updata_binary_file()?;
    if !updata_bin.exists() {
        return Ok(false);
    }

    let mut command = Command::new(updata_bin);
    let _ = notification;
    let _child = command.spawn()?;
    Ok(true)
}

pub fn spawn_helper_script(
    helper_name: &str,
    args: &[&str],
    current_dir: Option<&Path>,
) -> Result<bool> {
    let script = path_utils::helper_script_file(helper_name)?;
    if !script.exists() {
        return Ok(false);
    }

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(script.as_os_str());
        cmd
    };

    #[cfg(not(target_os = "windows"))]
    let mut command = {
        let mut cmd = Command::new("sh");
        cmd.arg(script.as_os_str());
        cmd
    };

    if let Some(dir) = current_dir {
        command.current_dir(dir);
    }
    for arg in args {
        command.arg(arg);
    }

    let _child = command.spawn()?;
    Ok(true)
}

fn fetch_latest_release() -> Result<Option<UpdateNotification>> {
    let payload = match fetch_latest_release_payload()? {
        Some(payload) => payload,
        None => return Ok(None),
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

fn fetch_latest_release_payload() -> Result<Option<ReleaseResponse>> {
    let client = Client::builder().timeout(Duration::from_secs(8)).build()?;

    let mut request = client
        .get(GITHUB_API_LATEST)
        .header(USER_AGENT, "tui-game-updater")
        .header(ACCEPT, "application/vnd.github+json");

    if !GITHUB_TOKEN.is_empty() {
        request = request.header(AUTHORIZATION, format!("Bearer {GITHUB_TOKEN}"));
    }

    let response = match request.send() {
        Ok(response) => response,
        Err(_) => return Ok(None),
    };

    if !response.status().is_success() {
        return Ok(None);
    }

    let payload = match response.json::<ReleaseResponse>() {
        Ok(payload) => payload,
        Err(_) => return Ok(None),
    };

    Ok(Some(payload))
}

fn parse_version_segments(version: &str) -> Option<Vec<u64>> {
    let clean = version.trim().trim_start_matches(['v', 'V']);
    if clean.is_empty() {
        return None;
    }

    let mut out = Vec::new();
    for part in clean.split('.') {
        if part.is_empty() || !part.chars().all(|ch| ch.is_ascii_digit()) {
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

    for idx in 0..max_len {
        let av = *a.get(idx).unwrap_or(&0);
        let bv = *b.get(idx).unwrap_or(&0);
        match av.cmp(&bv) {
            Ordering::Equal => {}
            non_eq => return Some(non_eq),
        }
    }

    Some(Ordering::Equal)
}
