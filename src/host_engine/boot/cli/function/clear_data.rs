//! CLI 功能函数模块 - 清空数据

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::host_engine::boot::cli::language;

/// 命令执行结果类型
type CommandResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 执行清空数据命令
/// 清空整个 data 目录（包含所有游戏存档、配置等）
pub fn execute() -> CommandResult<()> {
    let root_dir = root_dir();
    let data_dir = root_dir.join("data");

    // 显示清空数据对话框标题
    println!("{}", language::text(&language::CLI_CLEAR_DATA_TITLE));
    // 显示数据目录路径
    println!(
        "{}",
        language::format_text(
            &language::CLI_CLEAR_DATA_DATA,
            &[("path", &data_dir.display().to_string())],
        )
    );
    // 显示警告信息
    println!("{}", language::text(&language::CLI_CLEAR_DATA_WARN));

    // 等待用户确认
    if !confirm() {
        println!("{}", language::text(&language::CLI_CLEAR_DATA_NO));
        return Ok(());
    }

    // 清空数据目录
    clear_dir(&data_dir)?;

    // 显示成功信息
    println!("{}", language::text(&language::CLI_CLEAR_DATA_YES));
    Ok(())
}

/// 等待用户输入确认（输入 'y' 表示确认）
fn confirm() -> bool {
    let mut input = String::new();
    io::stdin().read_line(&mut input).is_ok() && input.trim().eq_ignore_ascii_case("y")
}

/// 清空指定目录（删除后重建）
fn clear_dir(path: &Path) -> CommandResult<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

/// 获取程序运行根目录（可执行文件所在目录或当前目录）
fn root_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}
