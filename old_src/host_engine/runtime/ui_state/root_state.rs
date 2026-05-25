//! UI Home 根状态

use mlua::{Lua, Table};

/// Home 页面选项数量。
pub const HOME_OPTION_COUNT: i64 = 5;

/// Home 页面确认动作。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HomeConfirmAction {
    GameList,
    ContinueGame,
    Setting,
    About,
}

/// Home 页面宿主根状态。
#[derive(Clone, Debug)]
pub struct HomeRootState {
    pub language: Vec<(String, String)>,
    pub select: i64,
    pub version: String,
    pub continue_items: Vec<(String, String)>,
}

impl HomeRootState {
    /// 规范化当前选项。
    pub fn normalize_select(&mut self) {
        self.select = normalize_home_select(self.select);
    }

    /// 选中项转为确认动作。
    pub fn confirm_action(&self) -> HomeConfirmAction {
        match normalize_home_select(self.select) {
            1 => HomeConfirmAction::GameList,
            2 => HomeConfirmAction::ContinueGame,
            3 => HomeConfirmAction::Setting,
            4 => HomeConfirmAction::About,
            _ => HomeConfirmAction::GameList,
        }
    }

    /// 转为 Lua root_state 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("language", pairs_to_table(lua, &self.language)?)?;
        table.set("select", normalize_home_select(self.select))?;
        table.set("version", self.version.as_str())?;
        table.set("continue", pairs_to_table(lua, &self.continue_items)?)?;
        Ok(table)
    }
}

/// 将 Home 选项限制在 1-5。
pub fn normalize_home_select(select: i64) -> i64 {
    select.clamp(1, HOME_OPTION_COUNT)
}

fn pairs_to_table(lua: &Lua, pairs: &[(String, String)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (key, value) in pairs {
        table.set(key.as_str(), value.as_str())?;
    }
    Ok(table)
}
