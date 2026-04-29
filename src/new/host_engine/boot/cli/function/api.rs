//! CLI 功能函数模块 - API 版本显示

use crate::host_engine::boot::cli::language;
use crate::host_engine::constant::API_VERSION;

/// 命令执行结果类型
type CommandResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 执行 API 版本显示命令
/// 输出当前宿主程序支持的 API 版本号
pub fn execute() -> CommandResult<()> {
    let api_version = API_VERSION.to_string();
    println!(
        "{}",
        language::format_text(&language::CLI_API, &[("api_version", &api_version)])
    );

    Ok(())
}
