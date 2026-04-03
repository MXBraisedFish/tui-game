use mlua::Lua;

use crate::core::screen::Canvas;

pub fn install_render_api(_lua: &Lua) -> mlua::Result<()> {
    Ok(())
}

pub fn measure_text(text: &str) -> (u16, u16) {
    Canvas::measure_text(text)
}
