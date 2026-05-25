//! Panic 崩溃日志记录
//!
//! 本模块只负责 panic 信息格式化、路径发现和写入，不负责安装 hook 或恢复终端。

use crate::host_engine::boot::environment::data_dirs;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::panic::PanicHookInfo;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Local;
use once_cell::sync::Lazy;
use serde::Serialize;

const CRASH_LOG_PATH: &str = "data/log/tui_crash.txt";

type CrashLogResult<T> = Result<T, Box<dyn std::error::Error>>;

static CURRENT_PHASE: AtomicU8 = AtomicU8::new(CrashPhase::Boot as u8);
static RUNTIME_INFO: Lazy<Mutex<RuntimeCrashInfo>> =
    Lazy::new(|| Mutex::new(RuntimeCrashInfo::default()));

/// 宿主当前阶段，用于定位 panic 发生阶段。
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CrashPhase {
    Boot = 0,
    Runtime = 1,
    Shutdown = 2,
}

/// panic 发生时可附加的运行期上下文。
#[derive(Debug, Clone, Default, Serialize)]
pub struct RuntimeCrashInfo {
    pub active_game_uid: Option<String>,
    pub active_ui_page: Option<String>,
    pub active_overlay_uid: Option<String>,
    pub lua_scope: Option<String>,
}

/// 单条崩溃日志。以 JSON line 形式追加到 `data/log/tui_crash.txt`。
#[derive(Debug, Serialize)]
pub struct CrashLog {
    pub timestamp: u64,
    pub datetime: String,
    pub app_version: String,
    pub platform: String,
    pub panic_message: String,
    pub location: String,
    pub thread_name: Option<String>,
    pub phase: String,
    pub backtrace: Option<String>,
    pub runtime_info: RuntimeCrashInfo,
}

/// 记录 panic 信息。
pub fn record_panic(info: &PanicHookInfo<'_>) -> CrashLogResult<()> {
    let crash_log = CrashLog {
        timestamp: current_timestamp_secs(),
        datetime: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        panic_message: panic_payload_text(info),
        location: panic_location(info),
        thread_name: std::thread::current().name().map(String::from),
        phase: current_phase().to_string(),
        backtrace: capture_backtrace(),
        runtime_info: current_runtime_info(),
    };

    write_crash_log(&crash_log)?;
    Ok(())
}

/// 更新当前宿主阶段。
pub fn set_phase(phase: CrashPhase) {
    CURRENT_PHASE.store(phase as u8, Ordering::Relaxed);
}

/// 更新运行期崩溃上下文。
pub fn set_runtime_info(info: RuntimeCrashInfo) {
    if let Ok(mut runtime_info) = RUNTIME_INFO.lock() {
        *runtime_info = info;
    }
}

/// 获取宿主根目录。优先使用可执行文件所在目录，失败时退回当前目录。
/// 追加写入 JSON line 崩溃日志。
fn write_crash_log(log: &CrashLog) -> io::Result<()> {
    let log_path = data_dirs::root_dir().join(CRASH_LOG_PATH);
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    let log_line = serde_json::to_string(log).map_err(io::Error::other)?;
    writeln!(log_file, "{log_line}")?;
    log_file.flush()
}

fn current_phase() -> &'static str {
    match CURRENT_PHASE.load(Ordering::Relaxed) {
        0 => "Boot",
        1 => "Runtime",
        2 => "Shutdown",
        _ => "Unknown",
    }
}

fn current_runtime_info() -> RuntimeCrashInfo {
    RUNTIME_INFO
        .lock()
        .map(|runtime_info| runtime_info.clone())
        .unwrap_or_default()
}

fn panic_location(info: &PanicHookInfo<'_>) -> String {
    info.location()
        .map(|location| {
            format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        })
        .unwrap_or_else(|| "unknown location".to_string())
}

fn panic_payload_text(info: &PanicHookInfo<'_>) -> String {
    info.payload()
        .downcast_ref::<&str>()
        .map(|payload| (*payload).to_string())
        .or_else(|| info.payload().downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic payload".to_string())
}

fn capture_backtrace() -> Option<String> {
    let backtrace = std::backtrace::Backtrace::force_capture();
    match backtrace.status() {
        std::backtrace::BacktraceStatus::Captured => Some(backtrace.to_string()),
        _ => None,
    }
}

fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
