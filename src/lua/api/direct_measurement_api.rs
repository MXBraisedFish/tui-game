use crossterm::terminal;
use mlua::{Lua, Value, Variadic};

use crate::core::screen::Canvas;
use crate::lua::api::common;

pub(crate) fn install(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();

    globals.set(
        "get_text_size",
        lua.create_function(|_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 1)?;
            let text = common::expect_string_arg(&args, 0, "text")?;
            let (width, height) = Canvas::measure_text(&text);
            Ok((i64::from(width), i64::from(height)))
        })?,
    )?;

    globals.set(
        "get_text_width",
        lua.create_function(|_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 1)?;
            let text = common::expect_string_arg(&args, 0, "text")?;
            let (width, _) = Canvas::measure_text(&text);
            Ok(i64::from(width))
        })?,
    )?;

    globals.set(
        "get_text_height",
        lua.create_function(|_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 1)?;
            let text = common::expect_string_arg(&args, 0, "text")?;
            let (_, height) = Canvas::measure_text(&text);
            Ok(i64::from(height))
        })?,
    )?;

    globals.set(
        "get_terminal_size",
        lua.create_function(|_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 0)?;
            let (width, height) = terminal::size().unwrap_or((80, 24));
            Ok((i64::from(width), i64::from(height)))
        })?,
    )?;

    Ok(())
}
