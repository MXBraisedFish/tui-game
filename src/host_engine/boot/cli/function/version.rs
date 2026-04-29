//! CLI 功能函数模块 - 版本检查与更新提示

use crate::host_engine::boot::cli::language;
use crate::host_engine::constant::{GITHUB_URL, HOST_VERSION, UPDATE_API_URL};

/// 命令执行结果类型
type CommandResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 执行版本检查命令
/// 显示当前版本，并从 GitHub 获取最新版本信息，提示是否有更新
pub fn execute() -> CommandResult<()> {
    // 显示当前版本
    println!(
        "{}",
        language::format_text(&language::CLI_VERSION_CURRENT, &[("version", HOST_VERSION)])
    );

    // 尝试获取最新版本
    match fetch_latest_version() {
        Ok(latest_version) => {
            // 显示最新版本
            println!(
                "{}",
                language::format_text(
                    &language::CLI_VERSION_LATEST,
                    &[("version", latest_version.as_str())],
                )
            );

            // 判断是否有可用更新
            if is_update_available(HOST_VERSION, latest_version.as_str()) {
                println!("{}", language::text(&language::CLI_VERSION_IS_UPDATE));
                println!(
                    "{}",
                    language::format_text(&language::CLI_VERSION_URL, &[("url", GITHUB_URL)])
                );
            } else {
                println!("{}", language::text(&language::CLI_VERSION_IS_NEW));
            }
        }
        Err(_) => {
            // 版本检查失败提示
            println!("{}", language::text(&language::CLI_VERSION_CHECK_FAILED));
        }
    }

    Ok(())
}

/// 从 GitHub API 获取最新版本号
/// 发送请求到 UPDATE_API_URL，解析返回的 JSON 中的 tag_name 字段
fn fetch_latest_version() -> CommandResult<String> {
    let response: serde_json::Value = reqwest::blocking::Client::new()
        .get(UPDATE_API_URL)
        .header("User-Agent", "tui-game")
        .send()?
        .error_for_status()?
        .json()?;

    Ok(response
        .get("tag_name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .trim_start_matches('v')
        .to_string())
}

/// 判断是否有可用更新
/// 比较当前版本与最新版本的**主版本号**和**次版本号**（前两位）
/// 忽略补丁版本号（第三位及之后）
fn is_update_available(current_version: &str, latest_version: &str) -> bool {
    let current_parts = parse_version(current_version);
    let latest_parts = parse_version(latest_version);

    // 取前两位比较，不足两位的补 0
    let current_major_minor = (
        current_parts.first().copied().unwrap_or(0),
        current_parts.get(1).copied().unwrap_or(0),
    );
    let latest_major_minor = (
        latest_parts.first().copied().unwrap_or(0),
        latest_parts.get(1).copied().unwrap_or(0),
    );

    latest_major_minor > current_major_minor
}

/// 解析版本号字符串为数字数组
/// 格式：`"v1.2.3"` → `[1, 2, 3]`，支持比较
fn parse_version(version: &str) -> Vec<u32> {
    version
        .trim_start_matches('v')
        .split('.')
        .map(|part| part.parse::<u32>().unwrap_or(0))
        .collect()
}
