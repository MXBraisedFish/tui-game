//! CLI 命令行解析模块
//! 负责解析命令行参数并分发到对应的功能函数

use super::{function, language};

/// CLI 命令处理结果类型
type CliResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 支持的 CLI 命令枚举
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CliCommand {
    /// 显示帮助信息
    Help,
    /// 显示 API 版本
    Api,
    /// 显示版本信息
    Version,
    /// 清空缓存目录
    ClearCache,
    /// 清空数据目录
    ClearData,
    /// 显示路径信息
    Path,
    /// 执行修复操作
    Fix,
}

/// CLI 命令处理入口
/// 返回值：true 表示命令已处理，程序应退出；false 表示无命令或需继续启动
pub fn handle_command() -> CliResult<bool> {
    // 获取第一个命令行参数（程序名后的第一个参数）
    let Some(raw_command) = std::env::args().nth(1) else {
        return Ok(false);
    };

    language::load();

    // 解析命令字符串为枚举
    let Some(command) = parse_command(&raw_command) else {
        eprintln!(
            "{}",
            language::format_text(&language::CLI_ERROR_UNKNOWN_ARG, &[("arg", &raw_command)])
        );
        eprintln!("{}", language::text(&language::CLI_ERROR_HELP));
        return Ok(true);
    };

    // 执行对应命令
    execute_command(command)?;
    Ok(true)
}

/// 将字符串解析为 CliCommand 枚举
fn parse_command(raw_command: &str) -> Option<CliCommand> {
    match raw_command.to_ascii_lowercase().as_str() {
        // 帮助命令
        "-h" | "-help" => Some(CliCommand::Help),
        // API 版本命令
        "-a" | "-api" => Some(CliCommand::Api),
        // 版本命令
        "-v" | "-version" => Some(CliCommand::Version),
        // 清空缓存命令
        "-cc" | "-clear-cache" => Some(CliCommand::ClearCache),
        // 清空数据命令
        "-cd" | "-clear-data" => Some(CliCommand::ClearData),
        // 显示路径命令
        "-p" | "-path" => Some(CliCommand::Path),
        // 修复命令
        "-f" | "-fix" => Some(CliCommand::Fix),
        _ => None,
    }
}

/// 根据命令枚举执行对应的功能函数
fn execute_command(command: CliCommand) -> CliResult<()> {
    match command {
        CliCommand::Help => function::help::execute()?,
        CliCommand::Api => function::api::execute()?,
        CliCommand::Version => function::version::execute()?,
        CliCommand::ClearCache => function::clear_cache::execute()?,
        CliCommand::ClearData => function::clear_data::execute()?,
        CliCommand::Path => function::path::execute()?,
        CliCommand::Fix => function::fix::execute()?,
    }

    Ok(())
}
