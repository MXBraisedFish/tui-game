//! Lua 表深拷贝

use mlua::{Lua, Table, Value};

/// 深拷贝 Lua 表。
pub fn deep_copy_table(lua: &Lua, table: &Table) -> mlua::Result<Table> {
    let copied_table = lua.create_table()?;
    for pair in table.clone().pairs::<Value, Value>() {
        let (key, value) = pair.map_err(mlua::Error::external)?;
        copied_table.set(deep_copy_value(lua, key)?, deep_copy_value(lua, value)?)?;
    }
    Ok(copied_table)
}

fn deep_copy_value(lua: &Lua, value: Value) -> mlua::Result<Value> {
    match value {
        Value::Table(table) => Ok(Value::Table(deep_copy_table(lua, &table)?)),
        other => Ok(other),
    }
}
