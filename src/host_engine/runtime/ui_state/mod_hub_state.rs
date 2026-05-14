//! Mod 设置中转页状态聚合。

use mlua::{Lua, Table, Value};

use crate::host_engine::boot::i18n;

const MOD_HUB_OPTION_COUNT: i64 = 3;

#[derive(Clone, Debug)]
pub struct ModHubUiState {
    pub root_state: ModHubRootState,
    pub lua_state: ModHubLuaState,
}

impl ModHubUiState {
    pub fn new() -> Self {
        let root_state = ModHubRootState::new();
        let lua_state = ModHubLuaState::from_root_state(&root_state);
        Self { root_state, lua_state }
    }

    pub fn reset_lua_state(&mut self) {
        self.root_state.refresh_language();
        self.lua_state = ModHubLuaState::from_root_state(&self.root_state);
    }

    pub fn refresh_language(&mut self) {
        self.root_state.refresh_language();
    }

    pub fn apply_lua_state(&mut self, lua_state: ModHubLuaState) -> ModHubLuaAction {
        self.lua_state = lua_state;
        self.root_state.select = normalize_select(self.lua_state.select);

        if self.lua_state.back {
            self.lua_state.back = false;
            return ModHubLuaAction::Back;
        }
        if self.lua_state.confirm {
            self.lua_state.confirm = false;
            return match self.root_state.select {
                1 => ModHubLuaAction::OpenGamePackList,
                2 => ModHubLuaAction::OpenSaverPackList,
                3 => ModHubLuaAction::OpenBossPackList,
                _ => ModHubLuaAction::None,
            };
        }
        ModHubLuaAction::None
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModHubLuaAction {
    None,
    Back,
    OpenGamePackList,
    OpenSaverPackList,
    OpenBossPackList,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModHubLuaState {
    pub select: i64,
    pub confirm: bool,
    pub back: bool,
}

impl ModHubLuaState {
    fn from_root_state(root_state: &ModHubRootState) -> Self {
        Self { select: root_state.select, confirm: false, back: false }
    }

    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("select", normalize_select(self.select))?;
        table.set("confirm", false)?;
        table.set("back", false)?;
        Ok(table)
    }

    pub fn from_lua_value(value: Value) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(table) => table,
            _ => return Err(mlua::Error::external("mod hub lua state must be returned as table")),
        };
        Ok(Self {
            select: normalize_select(table.get::<Option<i64>>("select")?.unwrap_or(1)),
            confirm: table.get::<Option<bool>>("confirm")?.unwrap_or(false),
            back: table.get::<Option<bool>>("back")?.unwrap_or(false),
        })
    }
}

#[derive(Clone, Debug)]
pub struct ModHubRootState {
    pub language: Vec<(String, String)>,
    pub select: i64,
}

impl ModHubRootState {
    fn new() -> Self {
        Self { language: mod_hub_language_pairs(), select: 1 }
    }

    fn refresh_language(&mut self) {
        self.language = mod_hub_language_pairs();
    }

    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("language", pairs_to_table(lua, &self.language)?)?;
        table.set("select", normalize_select(self.select))?;
        Ok(table)
    }
}

fn mod_hub_language_pairs() -> Vec<(String, String)> {
    let text = i18n::text();
    vec![
        ("MOD_PREV_OPTION".to_string(), text.key.mod_prev_option),
        ("MOD_NEXT_OPTION".to_string(), text.key.mod_next_option),
        ("MOD_LIST_OPTION1".to_string(), text.key.mod_list_option1),
        ("MOD_LIST_OPTION2".to_string(), text.key.mod_list_option2),
        ("MOD_LIST_OPTION3".to_string(), text.key.mod_list_option3),
        ("MOD_LIST_SELECT".to_string(), text.key.mod_hub_select),
        ("MOD_LIST_CONFIRM".to_string(), text.key.mod_hub_confirm),
        ("MOD_LIST_BACK".to_string(), text.key.mod_hub_back),
        ("MOD_LIST_TITLE".to_string(), text.key.mod_hub_title),
        ("MOD_HUB_GAME".to_string(), text.mod_hub.game),
        ("MOD_HUB_SAVER".to_string(), text.mod_hub.saver),
        ("MOD_HUB_BOSS".to_string(), text.mod_hub.boss),
    ]
}

fn pairs_to_table(lua: &Lua, pairs: &[(String, String)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (key, value) in pairs {
        table.set(key.as_str(), value.as_str())?;
    }
    Ok(table)
}

fn normalize_select(select: i64) -> i64 {
    if select < 1 { return MOD_HUB_OPTION_COUNT; }
    if select > MOD_HUB_OPTION_COUNT { return 1; }
    select
}
