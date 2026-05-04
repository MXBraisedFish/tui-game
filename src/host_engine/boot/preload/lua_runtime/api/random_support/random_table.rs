//! 随机数生成器 Lua 表构建

use mlua::{Lua, Table};

use super::random_store::RandomEntry;

/// 构建生成器信息表。
pub fn build_random_info_table(lua: &Lua, random_entry: &RandomEntry) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("id", random_entry.id.as_str())?;
    table.set("note", random_entry.note.as_str())?;
    table.set("seed", random_entry.seed.as_str())?;
    table.set("step", random_entry.step as i64)?;
    table.set("type", random_entry.kind.as_str())?;
    Ok(table)
}
