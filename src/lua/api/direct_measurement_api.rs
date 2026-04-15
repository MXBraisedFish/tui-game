use crossterm::terminal;
use mlua::Lua;

use crate::core::screen::Canvas;

pub(crate) fn install(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();

    globals.set(
        "get_text_size",
        lua.create_function(|_, text: String| {
            let (width, height) = Canvas::measure_text(&text);
            Ok((i64::from(width), i64::from(height)))
        })?,
    )?;

    globals.set(
        "get_text_width",
        lua.create_function(|_, text: String| {
            let (width, _) = Canvas::measure_text(&text);
            Ok(i64::from(width))
        })?,
    )?;

    globals.set(
        "get_text_height",
        lua.create_function(|_, text: String| {
            let (_, height) = Canvas::measure_text(&text);
            Ok(i64::from(height))
        })?,
    )?;

    globals.set(
        "get_terminal_size",
        lua.create_function(|_, ()| {
            let (width, height) = terminal::size().unwrap_or((80, 24));
            Ok((i64::from(width), i64::from(height)))
        })?,
    )?;

    Ok(())
}
