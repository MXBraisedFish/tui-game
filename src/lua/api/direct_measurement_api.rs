use crossterm::terminal;
use mlua::{Lua, Value, Variadic};

use crate::core::screen::Canvas;
use crate::lua::api::common;
use crate::lua::api::direct_debug_api;
use crate::lua::engine::RuntimeBridges;
use crate::utils::host_log;

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    {
        let bridges = bridges.clone();
        globals.set(
            "get_text_size",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let text = common::expect_string_arg(&args, 0, "text")?;
                let (width, height) = Canvas::measure_text(&text);
                write_measurement_warnings(&bridges, width, height);
                Ok((i64::from(width), i64::from(height)))
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "get_text_width",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let text = common::expect_string_arg(&args, 0, "text")?;
                let (width, height) = Canvas::measure_text(&text);
                write_measurement_warnings(&bridges, width, height);
                Ok(i64::from(width))
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "get_text_height",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let text = common::expect_string_arg(&args, 0, "text")?;
                let (width, height) = Canvas::measure_text(&text);
                write_measurement_warnings(&bridges, width, height);
                Ok(i64::from(height))
            })?,
        )?;
    }

    globals.set(
        "get_terminal_size",
        lua.create_function(move |_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 0)?;
            let (width, height) = terminal::size().map_err(|_| {
                host_log::append_host_error(
                    "host.exception.failed_to_get_terminal_size",
                    &[],
                );
                mlua::Error::external(crate::app::i18n::t_or(
                    "host.exception.failed_to_get_terminal_size",
                    "Failed to get terminal size, please check if the terminal environment is functioning properly.",
                ))
            })?;
            Ok((i64::from(width), i64::from(height)))
        })?,
    )?;

    Ok(())
}

fn write_measurement_warnings(bridges: &RuntimeBridges, width: u16, height: u16) {
    if width == 0 {
        let _ = direct_debug_api::write_log_line(
            bridges,
            &crate::app::i18n::t_or("debug.title.warning", "Warning"),
            &crate::app::i18n::t_or(
                "script.warning.computed_content_width_zero",
                "Computed content width is 0, which may cause display issues.",
            ),
        );
    }
    if height == 0 {
        let _ = direct_debug_api::write_log_line(
            bridges,
            &crate::app::i18n::t_or("debug.title.warning", "Warning"),
            &crate::app::i18n::t_or(
                "script.warning.computed_content_height_zero",
                "Computed content height is 0, which may cause display issues.",
            ),
        );
    }
}
