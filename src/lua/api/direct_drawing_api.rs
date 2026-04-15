use mlua::{Lua, Table, Value, Variadic};

use crate::core::screen::{Cell, ALIGN_CENTER, ALIGN_LEFT, ALIGN_NO_WRAP, ALIGN_RIGHT};
use crate::lua::engine::RuntimeBridges;

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    globals.set("ALIGN_LEFT", ALIGN_LEFT)?;
    globals.set("ALIGN_CENTER", ALIGN_CENTER)?;
    globals.set("ALIGN_RIGHT", ALIGN_RIGHT)?;

    {
        let bridges = bridges.clone();
        globals.set(
            "canvas_clear",
            lua.create_function(move |_, ()| {
                with_canvas(&bridges, |canvas| canvas.clear())?;
                Ok(())
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "canvas_draw_text",
            lua.create_function(move |_, (x, y, text, rest): (i64, i64, String, Variadic<Value>)| {
                let (fg, bg, align) = parse_draw_text_args(&rest);
                with_canvas(&bridges, |canvas| {
                    canvas.draw_text(to_u16(x), to_u16(y), &text, fg, bg, align);
                })?;
                Ok(())
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "canvas_fill_rect",
            lua.create_function(
                move |_, (x, y, width, height, rest): (i64, i64, i64, i64, Variadic<Value>)| {
                    let (fill_char, fg, bg) = parse_fill_rect_args(&rest);
                    with_canvas(&bridges, |canvas| {
                        canvas.fill_rect(
                            to_u16(x),
                            to_u16(y),
                            to_u16(width),
                            to_u16(height),
                            Cell {
                                ch: fill_char,
                                fg,
                                bg,
                                continuation: false,
                            },
                        );
                    })?;
                    Ok(())
                },
            )?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "canvas_eraser",
            lua.create_function(move |_, (x, y, width, height): (i64, i64, i64, i64)| {
                with_canvas(&bridges, |canvas| {
                    canvas.fill_rect(
                        to_u16(x),
                        to_u16(y),
                        to_u16(width),
                        to_u16(height),
                        Cell::default(),
                    );
                })?;
                Ok(())
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "canvas_border_rect",
            lua.create_function(
                move |_, (x, y, width, height, rest): (i64, i64, i64, i64, Variadic<Value>)| {
                    let (chars, fg, bg) = parse_border_rect_args(&rest)?;
                    with_canvas(&bridges, |canvas| {
                        draw_border_rect(
                            canvas,
                            to_u16(x),
                            to_u16(y),
                            to_u16(width),
                            to_u16(height),
                            &chars,
                            fg.clone(),
                            bg.clone(),
                        );
                    })?;
                    Ok(())
                },
            )?,
        )?;
    }

    Ok(())
}

#[derive(Default, Clone)]
struct BorderChars {
    top: Option<char>,
    top_right: Option<char>,
    right: Option<char>,
    bottom_right: Option<char>,
    bottom: Option<char>,
    bottom_left: Option<char>,
    left: Option<char>,
    top_left: Option<char>,
}

fn parse_draw_text_args(rest: &Variadic<Value>) -> (Option<String>, Option<String>, i64) {
    let fg = rest
        .first()
        .and_then(value_as_string)
        .filter(|value| !value.trim().is_empty());
    let bg = rest
        .get(1)
        .and_then(value_as_string)
        .filter(|value| !value.trim().is_empty());
    let align = match rest.get(2) {
        None => ALIGN_LEFT,
        Some(Value::Nil) => ALIGN_NO_WRAP,
        Some(Value::Integer(value)) => *value,
        Some(Value::Number(value)) => *value as i64,
        _ => ALIGN_LEFT,
    };
    (fg, bg, align)
}

fn with_canvas(
    bridges: &RuntimeBridges,
    f: impl FnOnce(&mut crate::core::screen::Canvas),
) -> mlua::Result<()> {
    let mut canvas = bridges
        .canvas
        .lock()
        .map_err(|_| mlua::Error::external("canvas poisoned"))?;
    f(&mut canvas);
    Ok(())
}

fn parse_fill_rect_args(rest: &Variadic<Value>) -> (char, Option<String>, Option<String>) {
    let fill_char = rest
        .first()
        .and_then(value_as_string)
        .and_then(first_char)
        .unwrap_or(' ');
    let fg = rest
        .get(1)
        .and_then(value_as_string)
        .filter(|value| !value.trim().is_empty());
    let bg = rest
        .get(2)
        .and_then(value_as_string)
        .filter(|value| !value.trim().is_empty());
    (fill_char, fg, bg)
}

fn parse_border_rect_args(
    rest: &Variadic<Value>,
) -> mlua::Result<(BorderChars, Option<String>, Option<String>)> {
    let chars = match rest.first() {
        Some(Value::Table(table)) => parse_border_chars(table)?,
        _ => BorderChars::default(),
    };
    let fg = rest
        .get(1)
        .and_then(value_as_string)
        .filter(|value| !value.trim().is_empty());
    let bg = rest
        .get(2)
        .and_then(value_as_string)
        .filter(|value| !value.trim().is_empty());
    Ok((chars, fg, bg))
}

fn parse_border_chars(table: &Table) -> mlua::Result<BorderChars> {
    Ok(BorderChars {
        top: get_optional_char(table, "top")?,
        top_right: get_optional_char(table, "top_right")?,
        right: get_optional_char(table, "right")?,
        bottom_right: get_optional_char(table, "bottom_right")?,
        bottom: get_optional_char(table, "bottom")?,
        bottom_left: get_optional_char(table, "bottom_left")?,
        left: get_optional_char(table, "left")?,
        top_left: get_optional_char(table, "top_left")?,
    })
}

fn get_optional_char(table: &Table, key: &str) -> mlua::Result<Option<char>> {
    let value: Value = table.get(key)?;
    Ok(value_as_string(&value).and_then(first_char))
}

fn draw_border_rect(
    canvas: &mut crate::core::screen::Canvas,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    chars: &BorderChars,
    fg: Option<String>,
    bg: Option<String>,
) {
    if width == 0 || height == 0 {
        return;
    }

    let max_x = x.saturating_add(width.saturating_sub(1));
    let max_y = y.saturating_add(height.saturating_sub(1));

    if width >= 2 {
        for col in x.saturating_add(1)..max_x {
            if let Some(ch) = chars.top {
                canvas.set_cell(col, y, make_cell(ch, fg.clone(), bg.clone()));
            }
            if let Some(ch) = chars.bottom {
                canvas.set_cell(col, max_y, make_cell(ch, fg.clone(), bg.clone()));
            }
        }
    }

    if height >= 2 {
        for row in y.saturating_add(1)..max_y {
            if let Some(ch) = chars.left {
                canvas.set_cell(x, row, make_cell(ch, fg.clone(), bg.clone()));
            }
            if let Some(ch) = chars.right {
                canvas.set_cell(max_x, row, make_cell(ch, fg.clone(), bg.clone()));
            }
        }
    }

    if let Some(ch) = chars.top_left {
        canvas.set_cell(x, y, make_cell(ch, fg.clone(), bg.clone()));
    }
    if let Some(ch) = chars.top_right {
        canvas.set_cell(max_x, y, make_cell(ch, fg.clone(), bg.clone()));
    }
    if let Some(ch) = chars.bottom_left {
        canvas.set_cell(x, max_y, make_cell(ch, fg.clone(), bg.clone()));
    }
    if let Some(ch) = chars.bottom_right {
        canvas.set_cell(max_x, max_y, make_cell(ch, fg, bg));
    }
}

fn make_cell(ch: char, fg: Option<String>, bg: Option<String>) -> Cell {
    Cell {
        ch,
        fg,
        bg,
        continuation: false,
    }
}

fn to_u16(value: i64) -> u16 {
    if value <= 0 {
        0
    } else if value >= i64::from(u16::MAX) {
        u16::MAX
    } else {
        value as u16
    }
}

fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::Nil => None,
        Value::String(value) => value.to_str().ok().map(|value| value.to_string()),
        Value::Integer(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::Boolean(value) => Some(value.to_string()),
        _ => None,
    }
}

fn first_char(value: String) -> Option<char> {
    value.chars().next()
}
