//! 直用式系统请求 API 公开

use mlua::{Lua, Value, Variadic};

use super::debug_support::lua_table_value;
use super::scope::ApiScope;
use super::validation::argument;
use crate::host_engine::boot::preload::lua_runtime::{HostLuaBridge, HostLuaMessage};

/// 安装系统请求 API。
pub fn install(lua: &Lua, api_scope: ApiScope, host_bridge: HostLuaBridge) -> mlua::Result<()> {
    let globals = lua.globals();

    if api_scope.allows_game_system_query() {
        install_get_launch_mode(lua, &globals, host_bridge.clone())?;
        install_get_best_score(lua, &globals, host_bridge.clone())?;
    }

    if api_scope.allows_common_system_request() {
        install_request_exit(lua, &globals, host_bridge.clone())?;
        install_request_skip_event_queue(lua, &globals, host_bridge.clone())?;
        install_request_clear_event_queue(lua, &globals, host_bridge.clone())?;
        install_request_render(lua, &globals, host_bridge.clone())?;
    }

    if api_scope.allows_game_storage_request() {
        install_request_save_best_score(lua, &globals, host_bridge.clone())?;
        install_request_save_game(lua, &globals, host_bridge)?;
    }

    Ok(())
}

fn install_get_launch_mode(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_launch_mode",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            let launch_mode = host_bridge.runtime_context().launch_mode;
            Ok(Value::String(lua.create_string(launch_mode.as_str())?))
        })?,
    )
}

fn install_get_best_score(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_best_score",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            let runtime_context = host_bridge.runtime_context();
            let Some(game_module) = runtime_context.current_game else {
                return Ok(Value::Nil);
            };
            match runtime_context.best_scores.get(game_module.uid.as_str()) {
                Some(best_score) => lua_table_value::json_to_lua_value(lua, best_score),
                None => Ok(Value::Nil),
            }
        })?,
    )
}

fn install_request_exit(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_empty_request(
        lua,
        globals,
        "request_exit",
        host_bridge,
        HostLuaMessage::ExitGame,
    )
}

fn install_request_skip_event_queue(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_empty_request(
        lua,
        globals,
        "request_skip_event_queue",
        host_bridge,
        HostLuaMessage::SkipEventQueue,
    )
}

fn install_request_clear_event_queue(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_empty_request(
        lua,
        globals,
        "request_clear_event_queue",
        host_bridge,
        HostLuaMessage::ClearEventQueue,
    )
}

fn install_request_render(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_empty_request(
        lua,
        globals,
        "request_render",
        host_bridge,
        HostLuaMessage::RenderNow,
    )
}

fn install_request_save_best_score(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_empty_request(
        lua,
        globals,
        "request_save_best_score",
        host_bridge,
        HostLuaMessage::SaveBestScore,
    )
}

fn install_request_save_game(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_empty_request(
        lua,
        globals,
        "request_save_game",
        host_bridge,
        HostLuaMessage::SaveGame,
    )
}

fn install_empty_request(
    lua: &Lua,
    globals: &mlua::Table,
    function_name: &str,
    host_bridge: HostLuaBridge,
    message: HostLuaMessage,
) -> mlua::Result<()> {
    globals.set(
        function_name,
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            host_bridge.push_message(message.clone());
            Ok(())
        })?,
    )
}
