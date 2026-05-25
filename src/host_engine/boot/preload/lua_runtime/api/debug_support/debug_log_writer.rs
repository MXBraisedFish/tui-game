//! 调试日志写入

use std::fs::{self, OpenOptions};
use std::io::Write;

use mlua::Value;

use super::debug_log_path;
use super::lua_stringify;
use crate::host_engine::boot::preload::game_modules::GameModuleSource;
use crate::host_engine::boot::preload::lua_runtime::{HostLuaBridge, LuaRuntimeConsumer};
use crate::host_engine::boot::preload::overlay_modules::OverlaySource;

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

/// 写入一行标准日志（不受调试开关控制，始终写入）。
pub fn write_std_log_line(
    host_bridge: &HostLuaBridge,
    title: &str,
    message: &str,
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
    writeln!(file, "[{title}] {message}").map_err(mlua::Error::external)
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

pub(crate) fn is_debug_enabled(host_bridge: &HostLuaBridge) -> bool {
    let runtime_context = host_bridge.runtime_context();
    match runtime_context.consumer {
        LuaRuntimeConsumer::OfficialUiPackage => true,
        LuaRuntimeConsumer::GamePackage => runtime_context
            .current_game
            .as_ref()
            .map(|game_module| {
                if game_module.source == GameModuleSource::Office {
                    return true;
                }
                runtime_context
                    .game_state
                    .get(game_module.uid.as_str())
                    .and_then(|state| state.get("debug"))
                    .and_then(|debug| debug.as_bool())
                    .unwrap_or(false)
            })
            .unwrap_or(false),
        LuaRuntimeConsumer::ScreensaverPackage => runtime_context
            .current_overlay
            .as_ref()
            .map(|overlay_package| {
                if overlay_package.source == OverlaySource::Office {
                    return true;
                }
                runtime_context
                    .screensaver_state
                    .get(overlay_package.uid.as_str())
                    .and_then(|state| state.get("debug"))
                    .and_then(|debug| debug.as_bool())
                    .unwrap_or(false)
            })
            .unwrap_or(false),
        LuaRuntimeConsumer::BossPackage => runtime_context
            .current_overlay
            .as_ref()
            .map(|overlay_package| {
                if overlay_package.source == OverlaySource::Office {
                    return true;
                }
                runtime_context
                    .boss_state
                    .get(overlay_package.uid.as_str())
                    .and_then(|state| state.get("debug"))
                    .and_then(|debug| debug.as_bool())
                    .unwrap_or(false)
            })
            .unwrap_or(false),
    }
}
