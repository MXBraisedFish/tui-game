//! 直用式调试 API 公开

use mlua::{Lua, Value, Variadic};

use super::debug_support::{debug_info_table, debug_key_table, debug_log_writer};
use super::scope::ApiScope;
use super::validation::argument;
use crate::host_engine::boot::preload::lua_runtime::HostLuaBridge;

const LOG_TITLE: &str = "日志";
const WARNING_TITLE: &str = "警告";
const ERROR_TITLE: &str = "异常";

/// 安装调试 API。
pub fn install(lua: &Lua, api_scope: ApiScope, host_bridge: HostLuaBridge) -> mlua::Result<()> {
    let globals = lua.globals();

    if api_scope.allows_debug_log() {
        install_debug_log(lua, &globals, host_bridge.clone())?;
        install_debug_warn(lua, &globals, host_bridge.clone())?;
        install_debug_error(lua, &globals, host_bridge.clone())?;
        install_debug_print(lua, &globals, host_bridge.clone())?;
        install_clear_debug_log(lua, &globals, host_bridge.clone())?;
    }

    if api_scope.allows_game_debug_info() {
        install_get_game_uid(lua, &globals, host_bridge.clone())?;
        install_get_game_info(lua, &globals, host_bridge.clone())?;
    }

    if api_scope.allows_key_query() {
        install_get_key(lua, &globals, host_bridge)?;
    }

    Ok(())
}

fn install_debug_log(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "debug_log",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            debug_log_writer::write_debug_line(&host_bridge, LOG_TITLE, &args[0])
        })?,
    )
}

fn install_debug_warn(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "debug_warn",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            debug_log_writer::write_debug_line(&host_bridge, WARNING_TITLE, &args[0])
        })?,
    )
}

fn install_debug_error(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "debug_error",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            debug_log_writer::write_debug_line(&host_bridge, ERROR_TITLE, &args[0])
        })?,
    )
}

fn install_debug_print(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "debug_print",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 2)?;
            let title = argument::expect_string_arg(&args, 0)?;
            debug_log_writer::write_debug_line(&host_bridge, title.as_str(), &args[1])
        })?,
    )
}

fn install_clear_debug_log(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "clear_debug_log",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            debug_log_writer::clear_debug_log(&host_bridge)
        })?,
    )
}

fn install_get_game_uid(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_game_uid",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            let runtime_context = host_bridge.runtime_context();
            let game_uid = runtime_context
                .current_game
                .map(|game_module| game_module.uid)
                .unwrap_or_default();
            Ok(Value::String(lua.create_string(game_uid)?))
        })?,
    )
}

fn install_get_game_info(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_game_info",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            let runtime_context = host_bridge.runtime_context();
            match runtime_context.current_game {
                Some(game_module) => Ok(Value::Table(debug_info_table::build_game_info_table(
                    lua,
                    &game_module,
                )?)),
                None => Ok(Value::Nil),
            }
        })?,
    )
}

fn install_get_key(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_key",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 0, 1)?;
            let requested_action = argument::expect_optional_string_arg(&args, 0)?;
            let runtime_context = host_bridge.runtime_context();
            Ok(Value::Table(debug_key_table::build_key_table(
                lua,
                runtime_context.current_game.as_ref(),
                &runtime_context.keybinds,
                requested_action.as_deref(),
            )?))
        })?,
    )
}
