//! UI Lua 返回状态

use mlua::{Lua, Table, Value};

use super::root_state::normalize_home_select;

/// Home 页面 Lua 运行状态。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HomeLuaState {
    pub select: i64,
    pub confirm: bool,
    pub exit: bool,
}

impl HomeLuaState {
    /// 创建新的 Lua state，确认状态总是重置为 false。
    pub fn new(select: i64) -> Self {
        Self {
            select: normalize_home_select(select),
            confirm: false,
            exit: false,
        }
    }

    /// 转为 Lua 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("select", normalize_home_select(self.select))?;
        table.set("confirm", self.confirm)?;
        table.set("exit", false)?;
        Ok(table)
    }

    /// 从 Lua 返回值解析。
    pub fn from_lua_value(value: Value) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(table) => table,
            _ => {
                return Err(mlua::Error::external(
                    "home lua state must be returned as table",
                ));
            }
        };
        let select = table.get::<Option<i64>>("select")?.unwrap_or(1);
        let confirm = table.get::<Option<bool>>("confirm")?.unwrap_or(false);
        let exit = table.get::<Option<bool>>("exit")?.unwrap_or(false);
        Ok(Self {
            select: normalize_home_select(select),
            confirm,
            exit,
        })
    }
}
