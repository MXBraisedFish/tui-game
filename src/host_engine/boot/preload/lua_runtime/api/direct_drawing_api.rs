//! 直用式内容绘制 API 公开

use mlua::{Lua, Value, Variadic};

use super::drawing_support::drawing_operation;
use super::drawing_support::drawing_parser;
use super::scope::ApiScope;
use super::validation::argument;
use crate::host_engine::boot::preload::lua_runtime::HostLuaBridge;

/// 安装内容绘制 API。
pub fn install(lua: &Lua, api_scope: ApiScope, host_bridge: HostLuaBridge) -> mlua::Result<()> {
    if !api_scope.allows_canvas_drawing() {
        return Ok(());
    }

    let globals = lua.globals();
    install_constants(&globals)?;
    install_canvas_clear(lua, &globals, host_bridge.clone())?;
    install_canvas_eraser(lua, &globals, host_bridge.clone())?;
    install_canvas_draw_text(lua, &globals, host_bridge.clone())?;
    install_canvas_fill_rect(lua, &globals, host_bridge.clone())?;
    install_canvas_border_rect(lua, &globals, host_bridge)?;

    Ok(())
}

fn install_constants(globals: &mlua::Table) -> mlua::Result<()> {
    globals.set("ALIGN_LEFT", drawing_parser::ALIGN_LEFT)?;
    globals.set("ALIGN_CENTER", drawing_parser::ALIGN_CENTER)?;
    globals.set("ALIGN_RIGHT", drawing_parser::ALIGN_RIGHT)?;

    globals.set("BOLD", drawing_parser::STYLE_BOLD)?;
    globals.set("ITALIC", drawing_parser::STYLE_ITALIC)?;
    globals.set("UNDERLINE", drawing_parser::STYLE_UNDERLINE)?;
    globals.set("STRIKE", drawing_parser::STYLE_STRIKE)?;
    globals.set("BLINK", drawing_parser::STYLE_BLINK)?;
    globals.set("REVERSE", drawing_parser::STYLE_REVERSE)?;
    globals.set("HIDDEN", drawing_parser::STYLE_HIDDEN)?;
    globals.set("DIM", drawing_parser::STYLE_DIM)?;

    Ok(())
}

fn install_canvas_clear(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "canvas_clear",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            host_bridge.with_canvas_state(|canvas_state| canvas_state.clear())
        })?,
    )
}

fn install_canvas_eraser(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "canvas_eraser",
        lua.create_function(move |_, args: Variadic<Value>| {
            let eraser_args = drawing_parser::parse_eraser_args(&args)?;
            host_bridge.with_canvas_state(|canvas_state| {
                drawing_operation::erase_rect(canvas_state, eraser_args);
            })
        })?,
    )
}

fn install_canvas_draw_text(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "canvas_draw_text",
        lua.create_function(move |_, args: Variadic<Value>| {
            let draw_text_args = drawing_parser::parse_draw_text_args(&args)?;
            host_bridge.with_canvas_state(|canvas_state| {
                drawing_operation::draw_text(canvas_state, draw_text_args);
            })
        })?,
    )
}

fn install_canvas_fill_rect(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "canvas_fill_rect",
        lua.create_function(move |_, args: Variadic<Value>| {
            let fill_rect_args = drawing_parser::parse_fill_rect_args(&args)?;
            host_bridge.with_canvas_state(|canvas_state| {
                drawing_operation::fill_rect(canvas_state, fill_rect_args);
            })
        })?,
    )
}

fn install_canvas_border_rect(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "canvas_border_rect",
        lua.create_function(move |_, args: Variadic<Value>| {
            let border_rect_args = drawing_parser::parse_border_rect_args(&args)?;
            host_bridge.with_canvas_state(|canvas_state| {
                drawing_operation::border_rect(canvas_state, border_rect_args);
            })
        })?,
    )
}
