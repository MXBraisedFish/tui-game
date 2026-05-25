//! CLI 功能函数模块 - 帮助信息显示

use crate::host_engine::boot::cli::language;

/// 命令执行结果类型
type CommandResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 执行帮助命令
/// 输出所有可用命令的说明信息
pub fn execute() -> CommandResult<()> {
    // 输出用法说明
    println!("{}", language::text(&language::CLI_HELP_USE));
    // 输出帮助头部
    println!("{}", language::text(&language::CLI_HELP_HEADER));
    // 输出无参数时的说明
    println!("{}", language::text(&language::CLI_HELP_NO_ARG));
    // 输出帮助命令说明
    println!("{}", language::text(&language::CLI_HELP_HELP));
    // 输出 API 命令说明
    println!("{}", language::text(&language::CLI_HELP_API));
    // 输出修复命令说明
    println!("{}", language::text(&language::CLI_HELP_FIX));
    // 输出路径命令说明
    println!("{}", language::text(&language::CLI_HELP_PATH));
    // 输出版本命令说明
    println!("{}", language::text(&language::CLI_HELP_VERSION));
    // 输出清空缓存命令说明
    println!("{}", language::text(&language::CLI_HELP_CLEAR_CACHE));
    // 输出清空数据命令说明
    println!("{}", language::text(&language::CLI_HELP_CLEAR_DATA));

    Ok(())
}
