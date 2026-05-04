//! UI 尺寸警告状态

use mlua::{Lua, Table};

use crate::host_engine::boot::i18n;
use crate::host_engine::boot::preload::init_environment::TerminalSize;

/// 尺寸警告模式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NeededSizeMode {
    Root,
    Game,
}

impl NeededSizeMode {
    /// 转为 Lua 字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Game => "game",
        }
    }
}

/// 尺寸警告 root_state。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NeededSizeRootState {
    pub actual: TerminalSize,
    pub needed: TerminalSize,
    pub mode: NeededSizeMode,
}

impl NeededSizeRootState {
    /// 转为 Lua root_state 表。
    pub fn to_lua_table(self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("language", language_table(lua)?)?;
        table.set("actual", size_table(lua, self.actual)?)?;
        table.set("needed", size_table(lua, self.needed)?)?;
        table.set("mode", self.mode.as_str())?;
        Ok(table)
    }
}

fn language_table(lua: &Lua) -> mlua::Result<Table> {
    let text = i18n::text();
    let table = lua.create_table()?;
    table.set("WARNING_SIZE_ACTUAL", text.warning.size_actual)?;
    table.set("WARNING_SIZE_NEEDED", text.warning.size_needed)?;
    table.set("WARNING_SIZE_HINT", text.warning.size_hint)?;
    table.set("WARNING_SIZE_ACTION_EXIT", text.warning.size_action_exit)?;
    table.set("WARNING_SIZE_ACTION_RETURN", text.warning.size_action_return)?;
    table.set("KEY_SIZE_RETURN", text.key.size_return)?;
    Ok(table)
}

fn size_table(lua: &Lua, terminal_size: TerminalSize) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("width", terminal_size.width)?;
    table.set("height", terminal_size.height)?;
    Ok(table)
}
