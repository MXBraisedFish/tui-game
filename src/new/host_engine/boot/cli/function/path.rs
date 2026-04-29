//! CLI 功能函数模块 - 显示安装路径

use std::path::{Path, PathBuf};

use crate::host_engine::boot::cli::language;

/// 命令执行结果类型
type CommandResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 执行路径显示命令
/// 输出宿主程序的安装目录路径
pub fn execute() -> CommandResult<()> {
    let install_dir = install_dir();
    println!(
        "{}",
        language::format_text(
            &language::CLI_PATH,
            &[("path", &install_dir.display().to_string())]
        )
    );

    Ok(())
}

/// 获取程序安装目录（可执行文件所在目录）
fn install_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}
