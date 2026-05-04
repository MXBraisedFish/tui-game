//! Lua 沙箱限制

use mlua::{Lua, Table, Value};

/// 安装 Lua 沙箱。
///
/// 禁止使用 Lua 官方危险库：
/// - `os`：整体替换为阻塞模块。
/// - `io`：整体替换为阻塞模块。
/// - `debug`：整体替换为阻塞模块。
///
/// TODO: 后续根据 API 安全策略决定是否额外限制 `package` / `require`。
pub fn install_sandbox(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();

    globals.set("os", blocked_module(lua, "os")?)?;
    globals.set("io", blocked_module(lua, "io")?)?;
    globals.set("debug", blocked_module(lua, "debug")?)?;

    Ok(())
}

fn blocked_module(lua: &Lua, module_name: &str) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let metatable = lua.create_table()?;
    let module_name = module_name.to_string();

    metatable.set(
        "__index",
        lua.create_function(move |lua, (_table, key): (Table, Value)| {
            let api_name = format_blocked_api_name(module_name.as_str(), key);
            blocked_builtin(lua, api_name)
        })?,
    )?;
    metatable.set(
        "__newindex",
        lua.create_function(
            |_, (_table, _key, _value): (Table, Value, Value)| -> mlua::Result<()> {
                Err(mlua::Error::external(
                    "sandbox-disabled Lua built-in API assignment is blocked",
                ))
            },
        )?,
    )?;

    table.set_metatable(Some(metatable))?;
    Ok(table)
}

fn blocked_builtin(lua: &Lua, api_name: String) -> mlua::Result<mlua::Function> {
    lua.create_function(move |_, _: mlua::Variadic<Value>| -> mlua::Result<()> {
        Err(mlua::Error::external(format!(
            "sandbox-disabled Lua built-in API is blocked: {api_name}"
        )))
    })
}

fn format_blocked_api_name(module_name: &str, key: Value) -> String {
    let suffix = match key {
        Value::String(value) => value
            .to_str()
            .map(|value| value.to_string())
            .unwrap_or_else(|_| "?".to_string()),
        Value::Integer(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        _ => "?".to_string(),
    };
    format!("{module_name}.{suffix}")
}
