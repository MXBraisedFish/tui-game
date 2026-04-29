// 测量 API，计算文本尺寸和终端尺寸。提供 get_text_size, get_text_width, get_text_height, get_terminal_size 函数，利用 Canvas::measure_text 实现宽字符支持，并对宽度/高度为零的情况发出警告

use crossterm::terminal; // 获取终端尺寸
use mlua::{Lua, Value, Variadic}; // Lua 类型

use crate::core::screen::Canvas; // 使用测量方法
use crate::lua::api::common; // 参数校验
use crate::lua::api::direct_debug_api; // 写警告日志
use crate::lua::engine::RuntimeBridges; // 运行时桥接
use crate::utils::host_log; // 日志

// 注册 API
pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    // 返回文本显示所需的宽度和高度（基于 Unicode 宽度）
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

    // 仅返回宽度
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

    // 仅返回高度
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

    // 返回终端当前尺寸（行列数）
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

// 当宽度或高度为 0 时，通过调试日志写入警告
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
