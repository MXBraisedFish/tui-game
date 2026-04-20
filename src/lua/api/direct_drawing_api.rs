use mlua::{Lua, Table, Value, Variadic};
use unicode_width::UnicodeWidthStr;

use crate::core::screen::{Cell, ALIGN_CENTER, ALIGN_LEFT, ALIGN_NO_WRAP, ALIGN_RIGHT};
use crate::lua::api::common;
use crate::lua::api::direct_debug_api;
use crate::lua::engine::RuntimeBridges;
use crate::utils::host_log;

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    globals.set("ALIGN_LEFT", ALIGN_LEFT)?;
    globals.set("ALIGN_CENTER", ALIGN_CENTER)?;
    globals.set("ALIGN_RIGHT", ALIGN_RIGHT)?;

    {
        let bridges = bridges.clone();
        globals.set(
            "canvas_clear",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                with_canvas(&bridges, |canvas| canvas.clear())?;
                Ok(())
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "canvas_draw_text",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_arg_count_range(&args, 3, 6)?;
                let x = common::expect_i64_arg(&args, 0, "x")?;
                let y = common::expect_i64_arg(&args, 1, "y")?;
                let text = common::expect_string_arg(&args, 2, "text")?;
                ensure_coordinate(x)?;
                ensure_coordinate(y)?;
                let (fg, bg, align) = parse_draw_text_args(&args[3..])?;
                with_canvas(&bridges, |canvas| {
                    warn_if_text_exceeds_canvas(&bridges, canvas, x, y, &text, align);
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
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_arg_count_range(&args, 4, 7)?;
                let x = common::expect_i64_arg(&args, 0, "x")?;
                let y = common::expect_i64_arg(&args, 1, "y")?;
                let width = common::expect_i64_arg(&args, 2, "width")?;
                let height = common::expect_i64_arg(&args, 3, "height")?;
                ensure_coordinate(x)?;
                ensure_coordinate(y)?;
                ensure_positive_size(width, height)?;
                let (fill_char, fg, bg) = parse_fill_rect_args(&bridges, &args[4..])?;
                with_canvas(&bridges, |canvas| {
                    warn_if_rect_exceeds_canvas(&bridges, canvas, x, y, width, height);
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
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "canvas_eraser",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 4)?;
                let x = common::expect_i64_arg(&args, 0, "x")?;
                let y = common::expect_i64_arg(&args, 1, "y")?;
                let width = common::expect_i64_arg(&args, 2, "width")?;
                let height = common::expect_i64_arg(&args, 3, "height")?;
                ensure_coordinate(x)?;
                ensure_coordinate(y)?;
                ensure_positive_size(width, height)?;
                with_canvas(&bridges, |canvas| {
                    warn_if_rect_exceeds_canvas(&bridges, canvas, x, y, width, height);
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
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_arg_count_range(&args, 4, 7)?;
                let x = common::expect_i64_arg(&args, 0, "x")?;
                let y = common::expect_i64_arg(&args, 1, "y")?;
                let width = common::expect_i64_arg(&args, 2, "width")?;
                let height = common::expect_i64_arg(&args, 3, "height")?;
                ensure_coordinate(x)?;
                ensure_coordinate(y)?;
                ensure_positive_size(width, height)?;
                let (chars, fg, bg) = parse_border_rect_args(&bridges, &args[4..])?;
                with_canvas(&bridges, |canvas| {
                    warn_if_rect_exceeds_canvas(&bridges, canvas, x, y, width, height);
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
            })?,
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

fn parse_draw_text_args(rest: &[Value]) -> mlua::Result<(Option<String>, Option<String>, i64)> {
    let fg = common::expect_optional_string_arg(rest, 0, "fg")?
        .filter(|value| !value.trim().is_empty());
    let bg = common::expect_optional_string_arg(rest, 1, "bg")?
        .filter(|value| !value.trim().is_empty());
    let align = match rest.get(2) {
        None => ALIGN_LEFT,
        Some(Value::Nil) => ALIGN_NO_WRAP,
        Some(Value::Integer(value)) => *value,
        Some(Value::Number(value)) => *value as i64,
        Some(value) => return Err(common::arg_type_error("align", "number", value)),
    };
    Ok((fg, bg, align))
}

fn with_canvas(
    bridges: &RuntimeBridges,
    f: impl FnOnce(&mut crate::core::screen::Canvas),
) -> mlua::Result<()> {
    let mut canvas = bridges
        .canvas
        .lock()
        .map_err(|_| {
            host_log::append_host_error(
                "host.exception.canvas_context_invalid",
                &[],
            );
            mlua::Error::external(crate::app::i18n::t_or(
                "host.exception.canvas_context_invalid",
                "Canvas context is invalid, unable to perform drawing operations.",
            ))
        })?;
    f(&mut canvas);
    Ok(())
}

fn parse_fill_rect_args(
    bridges: &RuntimeBridges,
    rest: &[Value],
) -> mlua::Result<(char, Option<String>, Option<String>)> {
    let fill_char = common::expect_optional_string_arg(rest, 0, "char")?
        .and_then(|value| normalize_fill_char_warning(bridges, &value))
        .unwrap_or(' ');
    let fg = common::expect_optional_string_arg(rest, 1, "fg")?
        .filter(|value| !value.trim().is_empty());
    let bg = common::expect_optional_string_arg(rest, 2, "bg")?
        .filter(|value| !value.trim().is_empty());
    Ok((fill_char, fg, bg))
}

fn parse_border_rect_args(
    bridges: &RuntimeBridges,
    rest: &[Value],
) -> mlua::Result<(BorderChars, Option<String>, Option<String>)> {
    let chars = match common::expect_optional_table_arg(rest, 0, "char_list")? {
        Some(table) => parse_border_chars(bridges, &table)?,
        None => BorderChars::default(),
    };
    let fg = common::expect_optional_string_arg(rest, 1, "fg")?
        .filter(|value| !value.trim().is_empty());
    let bg = common::expect_optional_string_arg(rest, 2, "bg")?
        .filter(|value| !value.trim().is_empty());
    Ok((chars, fg, bg))
}

fn parse_border_chars(bridges: &RuntimeBridges, table: &Table) -> mlua::Result<BorderChars> {
    Ok(BorderChars {
        top: get_optional_char(bridges, table, "top")?,
        top_right: get_optional_char(bridges, table, "top_right")?,
        right: get_optional_char(bridges, table, "right")?,
        bottom_right: get_optional_char(bridges, table, "bottom_right")?,
        bottom: get_optional_char(bridges, table, "bottom")?,
        bottom_left: get_optional_char(bridges, table, "bottom_left")?,
        left: get_optional_char(bridges, table, "left")?,
        top_left: get_optional_char(bridges, table, "top_left")?,
    })
}

fn get_optional_char(
    bridges: &RuntimeBridges,
    table: &Table,
    key: &str,
) -> mlua::Result<Option<char>> {
    let value: Value = table.get(key)?;
    Ok(match value {
        Value::Nil => None,
        Value::String(value) => value
            .to_str()
            .ok()
            .map(|value| value.to_string())
            .and_then(|value| normalize_fill_char_warning(bridges, &value)),
        other => return Err(common::arg_type_error(key, "string", &other)),
    })
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

fn ensure_coordinate(value: i64) -> mlua::Result<()> {
    if value >= 0 {
        Ok(())
    } else {
        let value_text = value.to_string();
        host_log::append_host_error(
            "host.exception.invalid_coordinate_parameter",
            &[("value", &value_text)],
        );
        Err(mlua::Error::external(
            crate::app::i18n::t_or(
                "host.exception.invalid_coordinate_parameter",
                "Invalid coordinate parameter: must be a positive integer, got `{value}`.",
            )
            .replace("{value}", &value_text),
        ))
    }
}

fn ensure_positive_size(width: i64, height: i64) -> mlua::Result<()> {
    if width > 0 && height > 0 {
        Ok(())
    } else {
        let width_text = width.to_string();
        let height_text = height.to_string();
        host_log::append_host_error(
            "host.exception.invalid_width_height_parameter",
            &[("width", &width_text), ("height", &height_text)],
        );
        Err(mlua::Error::external(
            crate::app::i18n::t_or(
                "host.exception.invalid_width_height_parameter",
                "Invalid width/height parameter: must be positive integers, got width `{width}`, height `{height}`.",
            )
            .replace("{width}", &width_text)
            .replace("{height}", &height_text),
        ))
    }
}

fn warn_if_rect_exceeds_canvas(
    bridges: &RuntimeBridges,
    canvas: &crate::core::screen::Canvas,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
) {
    let canvas_w = i64::from(canvas.width());
    let canvas_h = i64::from(canvas.height());
    if x + width > canvas_w || y + height > canvas_h {
        write_drawing_bounds_warning(bridges, canvas.width(), canvas.height(), x, y);
    }
}

fn warn_if_text_exceeds_canvas(
    bridges: &RuntimeBridges,
    canvas: &crate::core::screen::Canvas,
    x: i64,
    y: i64,
    text: &str,
    align: i64,
) {
    let canvas_w = i64::from(canvas.width());
    let canvas_h = i64::from(canvas.height());

    if align == ALIGN_NO_WRAP {
        let escaped = text.replace('\n', "\\n");
        let width = UnicodeWidthStr::width(escaped.as_str()) as i64;
        if x + width > canvas_w || y >= canvas_h {
            write_drawing_bounds_warning(bridges, canvas.width(), canvas.height(), x, y);
        }
        return;
    }

    let first_line = text.split('\n').next().unwrap_or("");
    let first_width = UnicodeWidthStr::width(first_line) as i64;

    for (row_offset, line) in text.split('\n').enumerate() {
        let line_width = UnicodeWidthStr::width(line) as i64;
        let start_x = match align {
            ALIGN_CENTER => x + ((first_width - line_width) / 2),
            ALIGN_RIGHT => x + (first_width - line_width),
            _ => x,
        };
        let draw_y = y + row_offset as i64;
        if start_x < 0 || draw_y < 0 || draw_y >= canvas_h || start_x + line_width > canvas_w {
            write_drawing_bounds_warning(bridges, canvas.width(), canvas.height(), x, y);
            return;
        }
    }
}

fn write_drawing_bounds_warning(
    bridges: &RuntimeBridges,
    width: u16,
    height: u16,
    x: i64,
    y: i64,
) {
    let width_text = width.to_string();
    let height_text = height.to_string();
    let x_text = x.to_string();
    let y_text = y.to_string();
    let message = crate::app::i18n::t_or(
        "script.warning.drawing_exceeds_canvas_boundaries",
        "Drawing content exceeds canvas boundaries: canvas size is {w} columns × {h} rows, drawing origin is ({x}, {y}).",
    )
    .replace("{w}", &width_text)
    .replace("{h}", &height_text)
    .replace("{x}", &x_text)
    .replace("{y}", &y_text);
    let _ = direct_debug_api::write_log_line(
        bridges,
        &crate::app::i18n::t_or("debug.title.warning", "Warning"),
        &message,
    );
}

fn normalize_fill_char_warning(bridges: &RuntimeBridges, value: &str) -> Option<char> {
    let first = value.chars().next()?;
    let length = value.chars().count();
    if length > 1 {
        write_fill_char_length_warning(bridges, length, value, first);
    }
    Some(first)
}

fn write_fill_char_length_warning(
    bridges: &RuntimeBridges,
    length: usize,
    source: &str,
    first: char,
) {
    let length_text = length.to_string();
    let char_text = first.to_string();
    let message = crate::app::i18n::t_or(
        "script.warning.fill_char_length_invalid",
        "The fill character length must be 0 or 1, but got {length} (string: {string}). The first character `{char}` will be automatically used as the fill content.",
    )
    .replace("{length}", &length_text)
    .replace("{string}", source)
    .replace("{char}", &char_text);
    let _ = direct_debug_api::write_log_line(
        bridges,
        &crate::app::i18n::t_or("debug.title.warning", "Warning"),
        &message,
    );
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
