//! 直用式内容尺寸计算 API 公开

use mlua::{Lua, Value, Variadic};

use super::measurement_support::{rich_text_measurement, text_measurement};
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
    install_get_rich_text_size(lua, &globals, host_bridge.clone())?;
    install_get_rich_text_width(lua, &globals, host_bridge.clone())?;
    install_get_rich_text_height(lua, &globals, host_bridge.clone())?;
    install_get_terminal_size(lua, &globals, host_bridge)?;

    Ok(())
}

fn install_get_rich_text_size(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_rich_text_size",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 1, 2)?;
            let rich_text = argument::expect_string_arg(&args, 0)?;
            let wrap_width = parse_optional_wrap_width(&args, 1)?;
            let runtime_context = host_bridge.runtime_context();
            let text_size = rich_text_measurement::measure_rich_text(
                rich_text.as_str(),
                wrap_width,
                &runtime_context,
            )?;
            Ok((i64::from(text_size.width), i64::from(text_size.height)))
        })?,
    )
}

fn install_get_rich_text_width(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_rich_text_width",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 1, 2)?;
            let rich_text = argument::expect_string_arg(&args, 0)?;
            let wrap_width = parse_optional_wrap_width(&args, 1)?;
            let runtime_context = host_bridge.runtime_context();
            let text_size = rich_text_measurement::measure_rich_text(
                rich_text.as_str(),
                wrap_width,
                &runtime_context,
            )?;
            Ok(i64::from(text_size.width))
        })?,
    )
}

fn install_get_rich_text_height(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_rich_text_height",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 1, 2)?;
            let rich_text = argument::expect_string_arg(&args, 0)?;
            let wrap_width = parse_optional_wrap_width(&args, 1)?;
            let runtime_context = host_bridge.runtime_context();
            let text_size = rich_text_measurement::measure_rich_text(
                rich_text.as_str(),
                wrap_width,
                &runtime_context,
            )?;
            Ok(i64::from(text_size.height))
        })?,
    )
}

fn install_get_text_size(lua: &Lua, globals: &mlua::Table) -> mlua::Result<()> {
    globals.set(
        "get_text_size",
        lua.create_function(|_, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 1, 2)?;
            let text = argument::expect_string_arg(&args, 0)?;
            let wrap_width = parse_optional_wrap_width(&args, 1)?;
            let text_size = text_measurement::measure_text(text.as_str(), wrap_width);
            Ok((i64::from(text_size.width), i64::from(text_size.height)))
        })?,
    )
}

fn install_get_text_width(lua: &Lua, globals: &mlua::Table) -> mlua::Result<()> {
    globals.set(
        "get_text_width",
        lua.create_function(|_, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 1, 2)?;
            let text = argument::expect_string_arg(&args, 0)?;
            let wrap_width = parse_optional_wrap_width(&args, 1)?;
            let text_size = text_measurement::measure_text(text.as_str(), wrap_width);
            Ok(i64::from(text_size.width))
        })?,
    )
}

fn install_get_text_height(lua: &Lua, globals: &mlua::Table) -> mlua::Result<()> {
    globals.set(
        "get_text_height",
        lua.create_function(|_, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 1, 2)?;
            let text = argument::expect_string_arg(&args, 0)?;
            let wrap_width = parse_optional_wrap_width(&args, 1)?;
            let text_size = text_measurement::measure_text(text.as_str(), wrap_width);
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

fn parse_optional_wrap_width(args: &Variadic<Value>, index: usize) -> mlua::Result<Option<u16>> {
    let Some(value) = argument::expect_optional_i64_arg(args, index)? else {
        return Ok(None);
    };
    if value == 0 {
        return Ok(None);
    }
    if value < 0 {
        return Err(mlua::Error::external(
            "wrap_width must be nil, 0, or a positive integer",
        ));
    }
    Ok(Some(u16::try_from(value).map_err(mlua::Error::external)?))
}
