//! Rust panic 钩子安装模块
//! 崩溃时优先恢复终端状态，再尽可能写入宿主日志

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::panic::PanicHookInfo;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Local;
use crossterm::cursor::Show;
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};

const HOST_LOG_PATH: &str = "data/log/tui_log.txt";

type PanicHookResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 安装宿主 panic 钩子
pub fn install() -> PanicHookResult<()> {
    let previous_handler = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        restore_terminal_state();

        let panic_message = format_panic_message(panic_info);
        if !append_panic_log(&panic_message) {
            eprintln!("{panic_message}");
        }

        previous_handler(panic_info);
    }));

    Ok(())
}

/// 尽可能恢复终端状态。panic 期间不允许继续抛错。
fn restore_terminal_state() {
    let _ = disable_raw_mode();

    let mut stdout = io::stdout();
    let _ = execute!(stdout, Show, DisableMouseCapture, LeaveAlternateScreen);
    let _ = stdout.flush();
    let _ = io::stderr().flush();
}

/// 格式化 panic 信息，包含 payload 与源码位置
fn format_panic_message(panic_info: &PanicHookInfo<'_>) -> String {
    let payload = panic_payload_text(panic_info);
    let location = panic_info
        .location()
        .map(|location| {
            format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        })
        .unwrap_or_else(|| "unknown location".to_string());

    format!("Program crashed: {payload} ({location})")
}

/// 提取 panic payload 文本
fn panic_payload_text(panic_info: &PanicHookInfo<'_>) -> String {
    panic_info
        .payload()
        .downcast_ref::<&str>()
        .map(|payload| (*payload).to_string())
        .or_else(|| panic_info.payload().downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic payload".to_string())
}

/// 若宿主日志已存在，则增量写入 panic 日志；否则返回 false 交由调用方打印
fn append_panic_log(panic_message: &str) -> bool {
    let log_path = root_dir().join(HOST_LOG_PATH);
    if !log_path.is_file() {
        return false;
    }

    append_log_line(&log_path, panic_message).is_ok()
}

/// 追加写入单行宿主日志
fn append_log_line(log_path: &Path, panic_message: &str) -> io::Result<()> {
    let timestamp = current_timestamp_secs();
    let time_text = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_line = format!("[{timestamp}][{time_text}] {panic_message}\n");

    let mut log_file = OpenOptions::new().append(true).open(log_path)?;
    log_file.write_all(log_line.as_bytes())?;
    log_file.flush()
}

/// 获取当前 Unix 秒级时间戳
fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 获取宿主根目录。优先使用可执行文件所在目录，失败时退回当前目录。
fn root_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}
