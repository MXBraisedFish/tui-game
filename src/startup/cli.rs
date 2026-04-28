// 处理命令行参数，供外部脚本查询版本号。属于"透传"功能——匹配到特殊参数时直接处理并通知退出，未匹配则继续正常启动

use anyhow::Result; // 错误处理

// 读取第一个命令行参数，匹配 -rv 或 -run-version（不区分大小写）时打印版本号并返回 Ok(true)；无参数或不匹配时返回 Ok(false)
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