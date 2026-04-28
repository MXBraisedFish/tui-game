// 后台版本更新检查。通过 GitHub API 查询最新 release 版本号，与当前版本比较，通过 channel 通知主循环是否有更新

use std::sync::mpsc::{self, Receiver}; // 创建后台线程和消息通道
use std::thread; // 启动后台线程
use std::time::Duration; // HTTP 请求超时

use anyhow::Result; // 错误处理
use serde::Deserialize; // 反序列化 GitHub API 响应

// GitHub 最新 release 的 API 地址
const LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest";

// GitHub API 响应结构
#[derive(Deserialize)]
struct LatestReleaseResponse {
    tag_name: String,
}

// 规范化版本标签，确保以 "v" 开头
pub fn normalize_tag(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with('v') || trimmed.starts_with('V') {
        format!("v{}", trimmed[1..].trim())
    } else {
        format!("v{}", trimmed)
    }
}

// 启动后台线程检查更新，返回接收端
pub fn spawn_update_check(current_version: String) -> Receiver<Option<String>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = check_latest_release(&current_version).ok().flatten();
        let _ = tx.send(result);
    });
    rx
}

// 查询 GitHub API 获取最新版本号，比较后返回新版本或 None
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

// 逐段比较版本号，判断远程是否更新
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

// 将版本号字符串解析为 Vec<u32>（如 "v1.2.3" → [1, 2, 3]）
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