//! Lua 沙箱限制

use mlua::{Lua, Value};

/// 安装 Lua 沙箱。
///
/// 采用白名单机制：通过 `Lua::new_with` 仅加载安全的标准库（math / utf8 / string / table）。
/// base 库由 mlua 始终加载，此处移除其中的危险函数 `dofile`、`loadfile` 和 `load`。
pub fn install_sandbox(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();

    globals.set("dofile", Value::Nil)?;
    globals.set("loadfile", Value::Nil)?;
    globals.set("load", Value::Nil)?;

    Ok(())
}
