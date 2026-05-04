//! 直用式内容尺寸计算 API 公开

use mlua::{Lua, Value, Variadic};

use super::measurement_support::text_measurement;
use super::scope::ApiScope;
use super::validation::argument;
use crate::host_engine::boot::preload::lua_runtime::HostLuaBridge;

/// 安装内容尺寸计算 API。
pub fn install(lua: &Lua, api_scope: ApiScope, host_bridge: HostLuaBridge) -> mlua::Result<()> {
    if !api_scope.allows_measurement() {
        return Ok(());
    }

    let globals = lua.globals();
    install_get_text_size(lua, &globals)?;
    install_get_text_width(lua, &globals)?;
    install_get_text_height(lua, &globals)?;
    install_get_terminal_size(lua, &globals, host_bridge)?;

    Ok(())
}

fn install_get_text_size(lua: &Lua, globals: &mlua::Table) -> mlua::Result<()> {
    globals.set(
        "get_text_size",
        lua.create_function(|_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let text = argument::expect_string_arg(&args, 0)?;
            let text_size = text_measurement::measure_text(text.as_str());
            Ok((i64::from(text_size.width), i64::from(text_size.height)))
        })?,
    )
}

fn install_get_text_width(lua: &Lua, globals: &mlua::Table) -> mlua::Result<()> {
    globals.set(
        "get_text_width",
        lua.create_function(|_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let text = argument::expect_string_arg(&args, 0)?;
            let text_size = text_measurement::measure_text(text.as_str());
            Ok(i64::from(text_size.width))
        })?,
    )
}

fn install_get_text_height(lua: &Lua, globals: &mlua::Table) -> mlua::Result<()> {
    globals.set(
        "get_text_height",
        lua.create_function(|_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let text = argument::expect_string_arg(&args, 0)?;
            let text_size = text_measurement::measure_text(text.as_str());
            Ok(i64::from(text_size.height))
        })?,
    )
}

fn install_get_terminal_size(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_terminal_size",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            let terminal_size = host_bridge.runtime_context().terminal_size;
            Ok((
                i64::from(terminal_size.width),
                i64::from(terminal_size.height),
            ))
        })?,
    )
}
