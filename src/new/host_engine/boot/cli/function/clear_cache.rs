//! CLI 功能函数模块 - 清空缓存

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::host_engine::boot::cli::language;

/// 命令执行结果类型
type CommandResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 执行清空缓存命令
/// 清空 data/cache 和 data/log 目录
pub fn execute() -> CommandResult<()> {
    let root_dir = root_dir();
    let cache_dir = root_dir.join("data").join("cache");
    let log_dir = root_dir.join("data").join("log");

    // 显示清空缓存对话框标题
    println!("{}", language::text(&language::CLI_CLEAR_CACHE_TITLE));
    // 显示缓存目录路径
    println!(
        "{}",
        language::format_text(
            &language::CLI_CLEAR_CACHE_CACHE,
            &[("path", &cache_dir.display().to_string())],
        )
    );
    // 显示日志目录路径
    println!(
        "{}",
        language::format_text(
            &language::CLI_CLEAR_CACHE_LOG,
            &[("path", &log_dir.display().to_string())],
        )
    );
    // 显示警告信息
    println!("{}", language::text(&language::CLI_CLEAR_CACHE_WARN));

    // 等待用户确认
    if !confirm() {
        println!("{}", language::text(&language::CLI_CLEAR_CACHE_NO));
        return Ok(());
    }

    // 清空缓存目录和日志目录
    clear_dir(&cache_dir)?;
    clear_dir(&log_dir)?;

    // 显示成功信息
    println!("{}", language::text(&language::CLI_CLEAR_CACHE_YES));
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
