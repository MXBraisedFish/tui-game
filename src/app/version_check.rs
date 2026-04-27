use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;

const LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest";

#[derive(Deserialize)]
struct LatestReleaseResponse {
    tag_name: String,
}

/// 规范化版本标签，确保以 "v" 开头。
pub fn normalize_tag(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with('v') || trimmed.starts_with('V') {
        format!("v{}", trimmed[1..].trim())
    } else {
        format!("v{}", trimmed)
    }
}

/// 启动后台线程，检查是否有新版本。
/// 返回接收端，当检查完成时发送 `Some(latest_tag)` 或 `None`。
pub fn spawn_update_check(current_version: String) -> Receiver<Option<String>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = check_latest_release(&current_version).ok().flatten();
        let _ = tx.send(result);
    });
    rx
}

/// 查询 GitHub 最新发布版本。
fn check_latest_release(current_version: &str) -> Result<Option<String>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let response = client
        .get(LATEST_RELEASE_API_URL)
        .header(reqwest::header::USER_AGENT, "tui-game")
        .send()?
        .error_for_status()?
        .json::<LatestReleaseResponse>()?;
    let latest_tag = normalize_tag(&response.tag_name);
    if is_remote_version_newer(current_version, &latest_tag) {
        Ok(Some(latest_tag))
    } else {
        Ok(None)
    }
}

/// 比较两个版本号字符串，判断远程版本是否更新。
fn is_remote_version_newer(current_version: &str, remote_version: &str) -> bool {
    let current = parse_version_segments(current_version);
    let remote = parse_version_segments(remote_version);
    let max_len = current.len().max(remote.len());
    for idx in 0..max_len {
        let current_part = *current.get(idx).unwrap_or(&0);
        let remote_part = *remote.get(idx).unwrap_or(&0);
        if remote_part > current_part {
            return true;
        }
        if remote_part < current_part {
            return false;
        }
    }
    false
}

/// 将版本号字符串解析为 `Vec<u32>`，例如 "v1.2.3" -> `[1, 2, 3]`。
fn parse_version_segments(version: &str) -> Vec<u32> {
    let trimmed = version
        .trim()
        .trim_start_matches('v')
        .trim_start_matches('V');
    trimmed
        .split('.')
        .map(|part| part.parse::<u32>().unwrap_or(0))
        .collect()
}