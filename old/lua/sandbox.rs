// Lua 沙箱安全限制，禁止 Mod 脚本执行危险操作。通过重写 os 表的部分函数，并用阻塞占位符替换 io 和 debug 全局表

use mlua::{Lua, Table, Value}; // Lua 虚拟机类型

use crate::app::i18n; // 国际化错误消息
use crate::utils::host_log; // 记录沙箱拦截日志

// 安装沙箱：替换 os.execute、os.remove、os.rename、os.exit；将 io 和 debug 替换为阻塞模块
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

// 创建一个总是抛出沙箱错误的 Lua 函数
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

// 创建一个代理表，对该表的任何字段访问都返回一个阻塞函数，实现完整模块的阻塞
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
