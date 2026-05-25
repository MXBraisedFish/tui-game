//! 辅助脚本执行环境

use mlua::{Lua, Table, Value};

/// 创建辅助脚本执行环境。
///
/// 环境会通过 `__index = _G` 读取宿主公开 API；脚本自身定义内容写入环境表，便于执行后返回。
pub fn create_function_environment(lua: &Lua) -> mlua::Result<Table> {
    let environment = lua.create_table()?;
    let metatable = lua.create_table()?;
    metatable.set("__index", lua.globals())?;
    environment.set_metatable(Some(metatable))?;
    Ok(environment)
}

/// 清理执行环境中的内部元字段。
pub fn clean_function_environment(environment: &Table) -> mlua::Result<()> {
    environment.set("_ENV", Value::Nil)?;
    Ok(())
}
