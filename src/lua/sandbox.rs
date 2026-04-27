/// Lua 沙箱安全限制，禁止 Mod 脚本执行危险操作
/// 业务逻辑：
/// 禁用系统调用
/// 禁止模块访问

use mlua::{Lua, Table, Value};

use crate::app::i18n;
use crate::utils::host_log;

pub fn install_sandbox(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();
    let os: Table = globals.get("os")?;

    for name in ["execute", "remove", "rename", "exit"] {
        os.set(name, blocked_builtin(lua, &format!("os.{name}"))?)?;
    }

    globals.set("io", blocked_module(lua, "io")?)?;
    globals.set("debug", blocked_module(lua, "debug")?)?;
    Ok(())
}

fn blocked_builtin(lua: &Lua, name: &str) -> mlua::Result<mlua::Function> {
    let name = name.to_string();
    lua.create_function(move |_, ()| -> mlua::Result<()> {
        host_log::append_host_error(
            "host.error.sandbox_builtin_blocked",
            &[("err", name.as_str())],
        );
        Err(mlua::Error::external(
            i18n::t_or(
                "host.error.sandbox_builtin_blocked",
                "Mod attempted to call sandbox-disabled Lua built-in API, blocked: {err}",
            )
            .replace("{err}", &name),
        ))
    })
}

fn blocked_module(lua: &Lua, module_name: &str) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let metatable = lua.create_table()?;
    let module_name = module_name.to_string();
    metatable.set(
        "__index",
        lua.create_function(move |lua, (_table, key): (Table, Value)| {
            let suffix = match key {
                Value::String(value) => match value.to_str() {
                    Ok(value) => value.to_string(),
                    Err(_) => "?".to_string(),
                },
                Value::Integer(value) => value.to_string(),
                _ => "?".to_string(),
            };
            blocked_builtin(lua, &format!("{}.{}", module_name, suffix))
        })?,
    )?;
    table.set_metatable(Some(metatable))?;
    Ok(table)
}
