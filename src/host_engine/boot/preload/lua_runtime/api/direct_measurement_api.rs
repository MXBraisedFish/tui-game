//! 直用式内容尺寸计算 API 公开

use mlua::{Lua, Value, Variadic};

use super::drawing_support::drawing_parser;
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
    globals.set("WINDOW", drawing_parser::WRAP_WINDOW)?;
    install_get_text_size(lua, &globals, host_bridge.clone())?;
    install_get_text_width(lua, &globals, host_bridge.clone())?;
    install_get_text_height(lua, &globals, host_bridge.clone())?;
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
            let wrap_options = resolve_measurement_wrap_options(
                drawing_parser::parse_optional_wrap_options(&args, 1)?,
                &host_bridge,
            );
            let runtime_context = host_bridge.runtime_context();
            let text_size = rich_text_measurement::measure_rich_text_with_options(
                rich_text.as_str(),
                &wrap_options,
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
            let wrap_options = resolve_measurement_wrap_options(
                drawing_parser::parse_optional_wrap_options(&args, 1)?,
                &host_bridge,
            );
            let runtime_context = host_bridge.runtime_context();
            let text_size = rich_text_measurement::measure_rich_text_with_options(
                rich_text.as_str(),
                &wrap_options,
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
            let wrap_options = resolve_measurement_wrap_options(
                drawing_parser::parse_optional_wrap_options(&args, 1)?,
                &host_bridge,
            );
            let runtime_context = host_bridge.runtime_context();
            let text_size = rich_text_measurement::measure_rich_text_with_options(
                rich_text.as_str(),
                &wrap_options,
                &runtime_context,
            )?;
            Ok(i64::from(text_size.height))
        })?,
    )
}

fn install_get_text_size(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_text_size",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 1, 2)?;
            let text = argument::expect_string_arg(&args, 0)?;
            let wrap_options = resolve_measurement_wrap_options(
                drawing_parser::parse_optional_wrap_options(&args, 1)?,
                &host_bridge,
            );
            let text_size =
                text_measurement::measure_text_with_options(text.as_str(), &wrap_options);
            Ok((i64::from(text_size.width), i64::from(text_size.height)))
        })?,
    )
}

fn install_get_text_width(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_text_width",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 1, 2)?;
            let text = argument::expect_string_arg(&args, 0)?;
            let wrap_options = resolve_measurement_wrap_options(
                drawing_parser::parse_optional_wrap_options(&args, 1)?,
                &host_bridge,
            );
            let text_size =
                text_measurement::measure_text_with_options(text.as_str(), &wrap_options);
            Ok(i64::from(text_size.width))
        })?,
    )
}

fn install_get_text_height(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_text_height",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 1, 2)?;
            let text = argument::expect_string_arg(&args, 0)?;
            let wrap_options = resolve_measurement_wrap_options(
                drawing_parser::parse_optional_wrap_options(&args, 1)?,
                &host_bridge,
            );
            let text_size =
                text_measurement::measure_text_with_options(text.as_str(), &wrap_options);
            Ok(i64::from(text_size.height))
        })?,
    )
}

fn resolve_measurement_wrap_options(
    wrap_options: drawing_parser::WrapOptions,
    host_bridge: &HostLuaBridge,
) -> drawing_parser::WrapOptions {
    let terminal_size = host_bridge.runtime_context().terminal_size;
    wrap_options.resolved(terminal_size.width, terminal_size.height)
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
