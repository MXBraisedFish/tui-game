//! 调试日志路径解析

use std::path::{Path, PathBuf};

use crate::host_engine::boot::preload::lua_runtime::host_bridge::{
    HostLuaBridge, LuaRuntimeConsumer,
};

const UI_LOG_FILE_NAME: &str = "ui_log.txt";

/// 获取当前调试日志路径。
pub fn debug_log_path(host_bridge: &HostLuaBridge) -> PathBuf {
    let runtime_context = host_bridge.runtime_context();
    let log_file_name = match runtime_context.consumer {
        LuaRuntimeConsumer::OfficialUiPackage => UI_LOG_FILE_NAME.to_string(),
        LuaRuntimeConsumer::GamePackage => runtime_context
            .current_game
            .map(|game_module| format!("{}.txt", game_module.uid))
            .unwrap_or_else(|| UI_LOG_FILE_NAME.to_string()),
    };

    root_dir().join("data/log").join(log_file_name)
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
