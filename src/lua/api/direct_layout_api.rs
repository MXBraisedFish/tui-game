use crossterm::terminal;
use mlua::{Lua, Value, Variadic};

use crate::lua::api::common;
use crate::utils::host_log;

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
        lua.create_function(|_, args: Variadic<Value>| {
            common::expect_arg_count_range(&args, 2, 3)?;
            let x_anchor = common::expect_i64_arg(&args, 0, "x_anchor")?;
            let width = common::expect_i64_arg(&args, 1, "cw")?;
            ensure_anchor(x_anchor)?;
            ensure_non_negative_width(width)?;
            let offset_x = common::expect_optional_i64_arg(&args, 2, "offset_x")?.unwrap_or(0);
            let (term_width, _) = terminal_size_i64();
            Ok(resolve_axis(x_anchor, term_width, width, offset_x))
        })?,
    )?;

    globals.set(
        "resolve_y",
        lua.create_function(|_, args: Variadic<Value>| {
            common::expect_arg_count_range(&args, 2, 3)?;
            let y_anchor = common::expect_i64_arg(&args, 0, "y_anchor")?;
            let height = common::expect_i64_arg(&args, 1, "ch")?;
            ensure_anchor(y_anchor)?;
            ensure_non_negative_height(height)?;
            let offset_y = common::expect_optional_i64_arg(&args, 2, "offset_y")?.unwrap_or(0);
            let (_, term_height) = terminal_size_i64();
            Ok(resolve_axis(y_anchor, term_height, height, offset_y))
        })?,
    )?;

    globals.set(
        "resolve_rect",
        lua.create_function(|_, args: Variadic<Value>| {
            common::expect_arg_count_range(&args, 4, 6)?;
            let x_anchor = common::expect_i64_arg(&args, 0, "x_anchor")?;
            let y_anchor = common::expect_i64_arg(&args, 1, "y_anchor")?;
            let width = common::expect_i64_arg(&args, 2, "width")?;
            let height = common::expect_i64_arg(&args, 3, "height")?;
            ensure_anchor(x_anchor)?;
            ensure_anchor(y_anchor)?;
            ensure_non_negative_width_height(width, height)?;
            let offset_x = common::expect_optional_i64_arg(&args, 4, "offset_x")?.unwrap_or(0);
            let offset_y = common::expect_optional_i64_arg(&args, 5, "offset_y")?.unwrap_or(0);
            let (term_width, term_height) = terminal_size_i64();
            let x = resolve_axis(x_anchor, term_width, width, offset_x);
            let y = resolve_axis(y_anchor, term_height, height, offset_y);
            Ok((x, y))
        })?,
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

fn ensure_anchor(anchor: i64) -> mlua::Result<()> {
    if matches!(anchor, ANCHOR_LEFT | ANCHOR_CENTER | ANCHOR_RIGHT) {
        Ok(())
    } else {
        host_log::append_host_error("host.exception.invalid_anchor_parameter", &[]);
        Err(mlua::Error::external(crate::app::i18n::t_or(
            "host.exception.invalid_anchor_parameter",
            "Invalid anchor parameter.",
        )))
    }
}

fn ensure_non_negative_width(width: i64) -> mlua::Result<()> {
    if width >= 0 {
        Ok(())
    } else {
        let width_text = width.to_string();
        host_log::append_host_error(
            "host.exception.invalid_width_parameter",
            &[("width", &width_text)],
        );
        Err(mlua::Error::external(
            crate::app::i18n::t_or(
                "host.exception.invalid_width_parameter",
                "Invalid width parameter: must be a non-negative integer, got width `{width}`.",
            )
            .replace("{width}", &width_text),
        ))
    }
}

fn ensure_non_negative_height(height: i64) -> mlua::Result<()> {
    if height >= 0 {
        Ok(())
    } else {
        let height_text = height.to_string();
        host_log::append_host_error(
            "host.exception.invalid_height_parameter",
            &[("height", &height_text)],
        );
        Err(mlua::Error::external(
            crate::app::i18n::t_or(
                "host.exception.invalid_height_parameter",
                "Invalid height parameter: must be a non-negative integer, got height `{height}`.",
            )
            .replace("{height}", &height_text),
        ))
    }
}

fn ensure_non_negative_width_height(width: i64, height: i64) -> mlua::Result<()> {
    if width >= 0 && height >= 0 {
        Ok(())
    } else {
        let width_text = width.to_string();
        let height_text = height.to_string();
        host_log::append_host_error(
            "host.exception.invalid_layout_width_height_parameter",
            &[("width", &width_text), ("height", &height_text)],
        );
        Err(mlua::Error::external(
            crate::app::i18n::t_or(
                "host.exception.invalid_layout_width_height_parameter",
                "Invalid width/height parameter: must be a non-negative integer, got width `{width}`, height `{height}`.",
            )
            .replace("{width}", &width_text)
            .replace("{height}", &height_text),
        ))
    }
}
