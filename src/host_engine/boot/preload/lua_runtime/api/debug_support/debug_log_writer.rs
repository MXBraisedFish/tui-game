//! 调试日志写入

use std::fs::{self, OpenOptions};
use std::io::Write;

use mlua::Value;

use super::debug_log_path;
use super::lua_stringify;
use crate::host_engine::boot::preload::lua_runtime::{HostLuaBridge, LuaRuntimeConsumer};

/// 写入一行调试日志。
pub fn write_debug_line(
    host_bridge: &HostLuaBridge,
    title: &str,
    message: &Value,
) -> mlua::Result<()> {
    if !is_debug_enabled(host_bridge) {
        return Ok(());
    }

    let path = debug_log_path::debug_log_path(host_bridge);
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir).map_err(mlua::Error::external)?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(mlua::Error::external)?;
    writeln!(
        file,
        "[{title}] {}",
        lua_stringify::stringify_value(message)
    )
    .map_err(mlua::Error::external)
}

/// 清空当前调试日志。
pub fn clear_debug_log(host_bridge: &HostLuaBridge) -> mlua::Result<()> {
    if !is_debug_enabled(host_bridge) {
        return Ok(());
    }

    let path = debug_log_path::debug_log_path(host_bridge);
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir).map_err(mlua::Error::external)?;
    }
    fs::write(path, "").map_err(mlua::Error::external)
}

fn is_debug_enabled(host_bridge: &HostLuaBridge) -> bool {
    let runtime_context = host_bridge.runtime_context();
    match runtime_context.consumer {
        LuaRuntimeConsumer::OfficialUiPackage => true,
        LuaRuntimeConsumer::GamePackage => runtime_context
            .current_game
            .as_ref()
            .and_then(|game_module| runtime_context.mod_state.get(game_module.uid.as_str()))
            .and_then(|state| state.get("debug"))
            .and_then(|debug| debug.as_bool())
            .unwrap_or(false),
        LuaRuntimeConsumer::ScreenPackage | LuaRuntimeConsumer::BossPackage => runtime_context
            .current_overlay
            .as_ref()
            .and_then(|overlay_package| runtime_context.overlay_state.get(overlay_package.uid.as_str()))
            .and_then(|state| state.get("debug"))
            .and_then(|debug| debug.as_bool())
            .unwrap_or(false),
    }
}
