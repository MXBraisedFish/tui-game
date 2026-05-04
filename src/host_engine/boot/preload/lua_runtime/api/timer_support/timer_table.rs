//! 计时器 Lua 表构建

use mlua::{Lua, Table};

use super::timer_store::TimerEntry;

/// 构建计时器信息表。
pub fn build_timer_info_table(lua: &Lua, timer: &mut TimerEntry) -> mlua::Result<Table> {
    let status = timer.status().as_str();
    let table = lua.create_table()?;
    table.set("id", timer.id.as_str())?;
    table.set("note", timer.note.as_str())?;
    table.set("status", status)?;
    table.set("elapsed", timer.elapsed_ms() as i64)?;
    table.set("remaining", timer.remaining_ms() as i64)?;
    table.set("duration", timer.duration_ms as i64)?;
    Ok(table)
}
