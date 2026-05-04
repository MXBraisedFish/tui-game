//! Lua 事件值构造

use mlua::{Lua, Table};

/// 传递给 handle_event(state, event) 的事件。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LuaEvent {
    Action { name: String },
    Key { name: String },
    Resize { width: u16, height: u16 },
    Tick { dt_ms: u64 },
}

impl LuaEvent {
    /// 转换为 Lua event 表。
    pub fn into_lua_table(self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        match self {
            Self::Action { name } => {
                table.set("type", "action")?;
                table.set("name", name)?;
            }
            Self::Key { name } => {
                table.set("type", "key")?;
                table.set("name", name)?;
            }
            Self::Resize { width, height } => {
                table.set("type", "resize")?;
                table.set("width", width)?;
                table.set("height", height)?;
            }
            Self::Tick { dt_ms } => {
                table.set("type", "tick")?;
                table.set("dt_ms", dt_ms)?;
            }
        }
        Ok(table)
    }
}
