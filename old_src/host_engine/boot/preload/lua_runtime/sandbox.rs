//! Lua 沙箱限制

use mlua::{Lua, MultiValue, Value, Variadic};

use super::api::debug_support::{debug_log_writer, lua_stringify};
use super::host_bridge::HostLuaBridge;

/// 安装 Lua 沙箱。
///
/// 采用白名单机制：通过 `Lua::new_with` 仅加载安全的标准库（math / utf8 / string / table）。
/// base 库由 mlua 始终加载，此处移除其中的危险函数 `dofile`、`loadfile` 和 `load`，
/// 并劫持 `print`、`assert`、`error` 重写为写入日志文件的宿主实现。
pub fn install_sandbox(lua: &Lua, host_bridge: &HostLuaBridge) -> mlua::Result<()> {
    let globals = lua.globals();

    globals.set("dofile", Value::Nil)?;
    globals.set("loadfile", Value::Nil)?;
    globals.set("load", Value::Nil)?;
    globals.set("loadstring", Value::Nil)?;
    globals.set("print", Value::Nil)?;
    globals.set("assert", Value::Nil)?;
    globals.set("error", Value::Nil)?;

    install_print(lua, &globals, host_bridge)?;
    install_assert(lua, &globals, host_bridge)?;
    install_error(lua, &globals, host_bridge)?;

    Ok(())
}

fn install_print(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: &HostLuaBridge,
) -> mlua::Result<()> {
    let host_bridge = host_bridge.clone();
    globals.set(
        "print",
        lua.create_function(move |_, args: Variadic<Value>| {
            if !debug_log_writer::is_debug_enabled(&host_bridge) {
                return Ok(());
            }
            let message = args
                .iter()
                .map(lua_stringify::stringify_value)
                .collect::<Vec<_>>()
                .join("\t");
            let _ = debug_log_writer::write_std_log_line(&host_bridge, "print", &message);
            Ok(())
        })?,
    )
}

fn install_assert(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: &HostLuaBridge,
) -> mlua::Result<()> {
    let host_bridge = host_bridge.clone();
    globals.set(
        "assert",
        lua.create_function(move |_lua, args: MultiValue| {
            let is_truthy = !matches!(
                args.front(),
                None | Some(&Value::Nil) | Some(&Value::Boolean(false))
            );

            if is_truthy {
                return Ok(args);
            }

            let error_msg = args
                .get(1)
                .map(|v| lua_stringify::stringify_value(v))
                .unwrap_or_else(|| "assertion failed!".to_string());

            if debug_log_writer::is_debug_enabled(&host_bridge) {
                let _ = debug_log_writer::write_std_log_line(&host_bridge, "assert", &error_msg);
            }
            Err(mlua::Error::RuntimeError(error_msg))
        })?,
    )
}

fn install_error(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: &HostLuaBridge,
) -> mlua::Result<()> {
    let host_bridge = host_bridge.clone();
    globals.set(
        "error",
        lua.create_function(move |_lua, args: MultiValue| -> mlua::Result<Value> {
            let error_msg = args
                .front()
                .map(|v| lua_stringify::stringify_value(v))
                .unwrap_or_default();

            if debug_log_writer::is_debug_enabled(&host_bridge) {
                let _ = debug_log_writer::write_std_log_line(&host_bridge, "error", &error_msg);
            }
            Err(mlua::Error::RuntimeError(error_msg))
        })?,
    )
}
