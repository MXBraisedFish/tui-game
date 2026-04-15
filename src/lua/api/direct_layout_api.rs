use crossterm::terminal;
use mlua::Lua;

const ANCHOR_LEFT: i64 = 0;
const ANCHOR_CENTER: i64 = 1;
const ANCHOR_RIGHT: i64 = 2;
const ANCHOR_TOP: i64 = 0;
const ANCHOR_MIDDLE: i64 = 1;
const ANCHOR_BOTTOM: i64 = 2;

pub(crate) fn install(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();

    globals.set("ANCHOR_LEFT", ANCHOR_LEFT)?;
    globals.set("ANCHOR_CENTER", ANCHOR_CENTER)?;
    globals.set("ANCHOR_RIGHT", ANCHOR_RIGHT)?;
    globals.set("ANCHOR_TOP", ANCHOR_TOP)?;
    globals.set("ANCHOR_MIDDLE", ANCHOR_MIDDLE)?;
    globals.set("ANCHOR_BOTTOM", ANCHOR_BOTTOM)?;

    globals.set(
        "resolve_x",
        lua.create_function(|_, (x_anchor, width, offset_x): (i64, i64, Option<i64>)| {
            let (term_width, _) = terminal_size_i64();
            Ok(resolve_axis(x_anchor, term_width, width, offset_x.unwrap_or(0)))
        })?,
    )?;

    globals.set(
        "resolve_y",
        lua.create_function(|_, (y_anchor, height, offset_y): (i64, i64, Option<i64>)| {
            let (_, term_height) = terminal_size_i64();
            Ok(resolve_axis(y_anchor, term_height, height, offset_y.unwrap_or(0)))
        })?,
    )?;

    globals.set(
        "resolve_rect",
        lua.create_function(
            |_, (x_anchor, y_anchor, width, height, offset_x, offset_y): (
                i64,
                i64,
                i64,
                i64,
                Option<i64>,
                Option<i64>,
            )| {
                let (term_width, term_height) = terminal_size_i64();
                let x = resolve_axis(x_anchor, term_width, width, offset_x.unwrap_or(0));
                let y = resolve_axis(y_anchor, term_height, height, offset_y.unwrap_or(0));
                Ok((x, y))
            },
        )?,
    )?;

    Ok(())
}

fn terminal_size_i64() -> (i64, i64) {
    let (width, height) = terminal::size().unwrap_or((80, 24));
    (i64::from(width), i64::from(height))
}

fn resolve_axis(anchor: i64, available: i64, content: i64, offset: i64) -> i64 {
    let base = if anchor == ANCHOR_CENTER {
        (available - content) / 2
    } else if anchor == 2 {
        available - content
    } else {
        0
    };
    base + offset
}
