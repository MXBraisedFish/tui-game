//! UI Setting Keybind 状态聚合

use mlua::{Lua, Table, Value};

use crate::host_engine::boot::i18n;

/// Setting Keybind 页面选项数量。
pub const KEYBIND_OPTION_COUNT: i64 = 3;

/// Setting Keybind 页面确认动作。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeybindConfirmAction {
    Global,
    System,
    Game,
}

/// Setting Keybind 页面宿主与 Lua 双层状态。
#[derive(Clone, Debug)]
pub struct KeybindUiState {
    pub root_state: KeybindRootState,
    pub lua_state: KeybindLuaState,
}

impl KeybindUiState {
    /// 创建初始 Keybind 状态。
    pub fn new() -> Self {
        let root_state = KeybindRootState {
            language: keybind_language_pairs(),
            select: 1,
        };
        let lua_state = KeybindLuaState::new(root_state.select);
        Self {
            root_state,
            lua_state,
        }
    }

    /// 进入 Keybind 页面时重置 Lua state，并从 root_state 同步 select。
    pub fn reset_lua_state(&mut self) {
        self.root_state.normalize_select();
        self.lua_state = KeybindLuaState::new(self.root_state.select);
    }

    /// 刷新 Keybind 页面语言文本。
    pub fn refresh_language(&mut self) {
        self.root_state.language = keybind_language_pairs();
    }

    /// 应用 Lua 返回状态。
    pub fn apply_lua_state(&mut self, lua_state: KeybindLuaState) -> KeybindLuaAction {
        self.lua_state = lua_state;
        self.root_state.select = self.lua_state.select;
        self.root_state.normalize_select();

        if self.lua_state.back {
            self.lua_state.back = false;
            return KeybindLuaAction::Back;
        }

        if self.lua_state.confirm {
            self.lua_state.confirm = false;
            return KeybindLuaAction::Confirm(self.root_state.confirm_action());
        }

        KeybindLuaAction::None
    }
}

/// Setting Keybind Lua 返回动作。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeybindLuaAction {
    None,
    Back,
    Confirm(KeybindConfirmAction),
}

/// Setting Keybind 页面 Lua 运行状态。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeybindLuaState {
    pub select: i64,
    pub confirm: bool,
    pub back: bool,
}

impl KeybindLuaState {
    /// 创建新的 Lua state，确认与返回状态总是重置为 false。
    pub fn new(select: i64) -> Self {
        Self {
            select: normalize_keybind_select(select),
            confirm: false,
            back: false,
        }
    }

    /// 转为 Lua 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("select", normalize_keybind_select(self.select))?;
        table.set("confirm", false)?;
        table.set("back", false)?;
        Ok(table)
    }

    /// 从 Lua 返回值解析。
    pub fn from_lua_value(value: Value) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(table) => table,
            _ => {
                return Err(mlua::Error::external(
                    "setting keybind lua state must be returned as table",
                ));
            }
        };
        let select = table.get::<Option<i64>>("select")?.unwrap_or(1);
        let confirm = table.get::<Option<bool>>("confirm")?.unwrap_or(false);
        let back = table.get::<Option<bool>>("back")?.unwrap_or(false);
        Ok(Self {
            select: normalize_keybind_select(select),
            confirm,
            back,
        })
    }
}

/// Setting Keybind 页面宿主根状态。
#[derive(Clone, Debug)]
pub struct KeybindRootState {
    pub language: Vec<(String, String)>,
    pub select: i64,
}

impl KeybindRootState {
    /// 规范化当前选项。
    pub fn normalize_select(&mut self) {
        self.select = normalize_keybind_select(self.select);
    }

    /// 选中项转为确认动作。
    pub fn confirm_action(&self) -> KeybindConfirmAction {
        match normalize_keybind_select(self.select) {
            1 => KeybindConfirmAction::Global,
            2 => KeybindConfirmAction::System,
            3 => KeybindConfirmAction::Game,
            _ => KeybindConfirmAction::Global,
        }
    }

    /// 转为 Lua root_state 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("language", pairs_to_table(lua, &self.language)?)?;
        table.set("select", normalize_keybind_select(self.select))?;
        Ok(table)
    }
}

/// 将 Keybind 选项限制在 1-3。
pub fn normalize_keybind_select(select: i64) -> i64 {
    select.clamp(1, KEYBIND_OPTION_COUNT)
}

fn keybind_language_pairs() -> Vec<(String, String)> {
    let text = i18n::text();
    vec![
        (
            "SETTING_KEYBIND_LIST_OPTION1".to_string(),
            text.key.setting_keybind_list_option1.to_string(),
        ),
        (
            "SETTING_KEYBIND_LIST_OPTION2".to_string(),
            text.key.setting_keybind_list_option2.to_string(),
        ),
        (
            "SETTING_KEYBIND_LIST_OPTION3".to_string(),
            text.key.setting_keybind_list_option3.to_string(),
        ),
        (
            "SETTING_KEYBIND_LIST_PREV_OPTION".to_string(),
            text.key.setting_keybind_list_prev_option.to_string(),
        ),
        (
            "SETTING_KEYBIND_LIST_NEXT_OPTION".to_string(),
            text.key.setting_keybind_list_next_option.to_string(),
        ),
        (
            "SETTING_KEYBIND_LIST_SELECT".to_string(),
            text.key.setting_keybind_list_select.to_string(),
        ),
        (
            "SETTING_KEYBIND_LIST_CONFIRM".to_string(),
            text.key.setting_keybind_list_confirm.to_string(),
        ),
        (
            "SETTING_KEYBIND_LIST_BACK".to_string(),
            text.key.setting_keybind_list_back.to_string(),
        ),
        (
            "SETTING_KEYBIND_LIST_TITLE".to_string(),
            text.key.setting_keybind_list_title.to_string(),
        ),
        (
            "SETTING_KEYBIND_LIST_GLOBAL".to_string(),
            text.setting_keybind.list_global.to_string(),
        ),
        (
            "SETTING_KEYBIND_LIST_SYSTEM".to_string(),
            text.setting_keybind.list_system.to_string(),
        ),
        (
            "SETTING_KEYBIND_LIST_GAME".to_string(),
            text.setting_keybind.list_game.to_string(),
        ),
    ]
}

fn pairs_to_table(lua: &Lua, pairs: &[(String, String)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (key, value) in pairs {
        table.set(key.as_str(), value.as_str())?;
    }
    Ok(table)
}
