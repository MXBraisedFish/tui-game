//! 声明式 API 契约校验

use mlua::{Function, Lua, Value};

/// 校验 Lua 全局函数存在。
pub fn require_function(lua: &Lua, function_name: &str) -> mlua::Result<()> {
    let _: Function = lua.globals().get(function_name)?;
    Ok(())
}

/// 校验声明式 API 返回了需要的值。
pub fn ensure_returned_value(value: &Value) -> mlua::Result<()> {
    if matches!(value, Value::Nil) {
        Err(mlua::Error::external(
            "callback API did not return the required value",
        ))
    } else {
        Ok(())
    }
}
