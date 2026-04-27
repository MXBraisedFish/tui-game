use anyhow::Result;

/// 处理命令行透传标志。
///
/// 当前支持：
/// - `-rv` 或 `-run-version`：打印版本号后退出。
///
/// 返回 `Ok(true)` 表示已处理并应立即退出程序，
/// 返回 `Ok(false)` 表示没有匹配的 CLI 指令，继续正常启动。
pub fn handle_cli_passthrough() -> Result<bool> {
    let arg = match std::env::args().nth(1) {
        Some(value) => value,
        None => return Ok(false),
    };
    if arg.eq_ignore_ascii_case("-rv") || arg.eq_ignore_ascii_case("-run-version") {
        let raw_version = env!("CARGO_PKG_VERSION");
        println!("v{}", raw_version.trim());
        return Ok(true);
    }
    Ok(false)
}