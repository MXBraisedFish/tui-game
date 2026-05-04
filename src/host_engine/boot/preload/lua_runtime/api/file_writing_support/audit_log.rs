//! 高风险写入审计日志

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Local;

use crate::host_engine::boot::preload::lua_runtime::HostLuaBridge;

/// 写入文件请求审计结果。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteAuditStatus {
    Allowed,
    Denied,
}

impl WriteAuditStatus {
    fn as_zh_text(self) -> &'static str {
        match self {
            Self::Allowed => "已被允许",
            Self::Denied => "已被拒绝",
        }
    }
}

/// 记录一次高风险写入请求。
pub fn append_write_request(
    host_bridge: &HostLuaBridge,
    api_name: &str,
    path: &Path,
    status: WriteAuditStatus,
) -> mlua::Result<()> {
    let runtime_context = host_bridge.runtime_context();
    let game_uid = runtime_context
        .current_game
        .as_ref()
        .map(|game_module| game_module.uid.as_str())
        .unwrap_or("unknown");
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_line = format!(
        "{game_uid} 于 {timestamp} 请求调用 {api_name}，路径：{}，{}。\n",
        path.display(),
        status.as_zh_text()
    );

    let log_path = root_dir().join("data/log/tui_log.txt");
    if let Some(parent_dir) = log_path.parent() {
        fs::create_dir_all(parent_dir).map_err(mlua::Error::external)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(mlua::Error::external)?;
    file.write_all(log_line.as_bytes())
        .map_err(mlua::Error::external)
}

fn root_dir() -> PathBuf {
    std::env::current_dir()
        .ok()
        .filter(|path| path.join("assets").exists() || path.join("Cargo.toml").exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(Path::to_path_buf))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}
