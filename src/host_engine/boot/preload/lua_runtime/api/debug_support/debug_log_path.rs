//! 调试日志路径解析

use crate::host_engine::boot::environment::data_dirs;
use std::path::PathBuf;

use crate::host_engine::boot::preload::lua_runtime::host_bridge::{
    HostLuaBridge, LuaRuntimeConsumer,
};

const FALLBACK_LOG_FILE_NAME: &str = "tui_log.txt";

/// 获取当前调试日志路径。
pub fn debug_log_path(host_bridge: &HostLuaBridge) -> PathBuf {
    let runtime_context = host_bridge.runtime_context();
    let log_file_name = match runtime_context.consumer {
        LuaRuntimeConsumer::GamePackage => runtime_context
            .current_game
            .map(|game_module| format!("{}.txt", game_module.uid))
            .unwrap_or_else(|| FALLBACK_LOG_FILE_NAME.to_string()),
        LuaRuntimeConsumer::ScreensaverPackage | LuaRuntimeConsumer::BossPackage => runtime_context
            .current_overlay
            .map(|overlay_package| format!("{}.txt", overlay_package.uid))
            .unwrap_or_else(|| FALLBACK_LOG_FILE_NAME.to_string()),
    };

    data_dirs::root_dir().join("data/log").join(log_file_name)
}
