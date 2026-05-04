//! 直用式布局定位计算 API 公开

use mlua::{Lua, Value, Variadic};

use super::layout_support::layout_anchor;
use super::layout_support::layout_parser;
use super::layout_support::layout_resolver;
use super::scope::ApiScope;
use crate::host_engine::boot::preload::lua_runtime::HostLuaBridge;

/// 安装布局定位计算 API。
pub fn install(lua: &Lua, api_scope: ApiScope, host_bridge: HostLuaBridge) -> mlua::Result<()> {
    if !api_scope.allows_layout() {
        return Ok(());
    }

    let globals = lua.globals();
    install_constants(&globals)?;
    install_resolve_x(lua, &globals, host_bridge.clone())?;
    install_resolve_y(lua, &globals, host_bridge.clone())?;
    install_resolve_rect(lua, &globals, host_bridge)?;

    Ok(())
}

fn install_constants(globals: &mlua::Table) -> mlua::Result<()> {
    globals.set("ANCHOR_LEFT", layout_anchor::ANCHOR_LEFT)?;
    globals.set("ANCHOR_CENTER", layout_anchor::ANCHOR_CENTER)?;
    globals.set("ANCHOR_RIGHT", layout_anchor::ANCHOR_RIGHT)?;
    globals.set("ANCHOR_TOP", layout_anchor::ANCHOR_TOP)?;
    globals.set("ANCHOR_MIDDLE", layout_anchor::ANCHOR_MIDDLE)?;
    globals.set("ANCHOR_BOTTOM", layout_anchor::ANCHOR_BOTTOM)?;
    Ok(())
}

fn install_resolve_x(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "resolve_x",
        lua.create_function(move |_, args: Variadic<Value>| {
            let resolve_args = layout_parser::parse_resolve_x_args(&args)?;
            let terminal_size = host_bridge.runtime_context().terminal_size;
            Ok(layout_resolver::resolve_x(
                i64::from(terminal_size.width),
                resolve_args,
            ))
        })?,
    )
}

fn install_resolve_y(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "resolve_y",
        lua.create_function(move |_, args: Variadic<Value>| {
            let resolve_args = layout_parser::parse_resolve_y_args(&args)?;
            let terminal_size = host_bridge.runtime_context().terminal_size;
            Ok(layout_resolver::resolve_y(
                i64::from(terminal_size.height),
                resolve_args,
            ))
        })?,
    )
}

fn install_resolve_rect(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "resolve_rect",
        lua.create_function(move |_, args: Variadic<Value>| {
            let resolve_args = layout_parser::parse_resolve_rect_args(&args)?;
            let terminal_size = host_bridge.runtime_context().terminal_size;
            Ok(layout_resolver::resolve_rect(
                i64::from(terminal_size.width),
                i64::from(terminal_size.height),
                resolve_args,
            ))
        })?,
    )
}
