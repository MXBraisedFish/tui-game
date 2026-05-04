//! UI Setting 状态聚合

use mlua::{Lua, Table, Value};

use crate::host_engine::boot::i18n;
use crate::host_engine::boot::preload::state_machine::SettingState;

/// Setting 页面选项数量。
pub const SETTING_OPTION_COUNT: i64 = 5;

/// Setting 页面确认动作。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingConfirmAction {
    Language,
    Keybind,
    ModList,
    Memory,
    Security,
}

impl SettingConfirmAction {
    /// 转为设置中层状态。
    pub fn to_setting_state(self) -> SettingState {
        match self {
            Self::Language => SettingState::Language,
            Self::Keybind => SettingState::Keybind,
            Self::ModList => SettingState::ModList,
            Self::Memory => SettingState::Memory,
            Self::Security => SettingState::Security,
        }
    }
}

/// Setting 页面宿主与 Lua 双层状态。
#[derive(Clone, Debug)]
pub struct SettingUiState {
    pub root_state: SettingRootState,
    pub lua_state: SettingLuaState,
}

impl SettingUiState {
    /// 创建初始 Setting 状态。
    pub fn new() -> Self {
        let root_state = SettingRootState {
            language: setting_language_pairs(),
            select: 1,
        };
        let lua_state = SettingLuaState::new(root_state.select);
        Self {
            root_state,
            lua_state,
        }
    }

    /// 进入 Setting 页面时重置 Lua state，并从 root_state 同步 select。
    pub fn reset_lua_state(&mut self) {
        self.root_state.normalize_select();
        self.lua_state = SettingLuaState::new(self.root_state.select);
    }

    /// 应用 Lua 返回状态。
    pub fn apply_lua_state(&mut self, lua_state: SettingLuaState) -> SettingLuaAction {
        self.lua_state = lua_state;
        self.root_state.select = self.lua_state.select;
        self.root_state.normalize_select();

        if self.lua_state.back {
            self.lua_state.back = false;
            return SettingLuaAction::Back;
        }

        if self.lua_state.confirm {
            self.lua_state.confirm = false;
            return SettingLuaAction::Confirm(self.root_state.confirm_action());
        }

        SettingLuaAction::None
    }
}

/// Setting Lua 返回动作。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingLuaAction {
    None,
    Back,
    Confirm(SettingConfirmAction),
}

/// Setting 页面 Lua 运行状态。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettingLuaState {
    pub select: i64,
    pub confirm: bool,
    pub back: bool,
}

impl SettingLuaState {
    /// 创建新的 Lua state，确认与返回状态总是重置为 false。
    pub fn new(select: i64) -> Self {
        Self {
            select: normalize_setting_select(select),
            confirm: false,
            back: false,
        }
    }

    /// 转为 Lua 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("select", normalize_setting_select(self.select))?;
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
                    "setting lua state must be returned as table",
                ));
            }
        };
        let select = table.get::<Option<i64>>("select")?.unwrap_or(1);
        let confirm = table.get::<Option<bool>>("confirm")?.unwrap_or(false);
        let back = table.get::<Option<bool>>("back")?.unwrap_or(false);
        Ok(Self {
            select: normalize_setting_select(select),
            confirm,
            back,
        })
    }
}

/// Setting 页面宿主根状态。
#[derive(Clone, Debug)]
pub struct SettingRootState {
    pub language: Vec<(String, String)>,
    pub select: i64,
}

impl SettingRootState {
    /// 规范化当前选项。
    pub fn normalize_select(&mut self) {
        self.select = normalize_setting_select(self.select);
    }

    /// 选中项转为确认动作。
    pub fn confirm_action(&self) -> SettingConfirmAction {
        match normalize_setting_select(self.select) {
            1 => SettingConfirmAction::Language,
            2 => SettingConfirmAction::Keybind,
            3 => SettingConfirmAction::ModList,
            4 => SettingConfirmAction::Memory,
            5 => SettingConfirmAction::Security,
            _ => SettingConfirmAction::Language,
        }
    }

    /// 转为 Lua root_state 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("language", pairs_to_table(lua, &self.language)?)?;
        table.set("select", normalize_setting_select(self.select))?;
        Ok(table)
    }
}

/// 将 Setting 选项限制在 1-5。
pub fn normalize_setting_select(select: i64) -> i64 {
    select.clamp(1, SETTING_OPTION_COUNT)
}

fn setting_language_pairs() -> Vec<(String, String)> {
    let text = i18n::text();
    vec![
        (
            "SETTING_PREV_OPTION".to_string(),
            text.key.setting_prev_option.to_string(),
        ),
        (
            "SETTING_NEXT_OPTION".to_string(),
            text.key.setting_next_option.to_string(),
        ),
        ("SETTING_SELECT".to_string(), text.key.setting_select.to_string()),
        (
            "SETTING_CONFIRM".to_string(),
            text.key.setting_confirm.to_string(),
        ),
        ("SETTING_OPTION1".to_string(), text.key.setting_option1.to_string()),
        ("SETTING_OPTION2".to_string(), text.key.setting_option2.to_string()),
        ("SETTING_OPTION3".to_string(), text.key.setting_option3.to_string()),
        ("SETTING_OPTION4".to_string(), text.key.setting_option4.to_string()),
        ("SETTING_OPTION5".to_string(), text.key.setting_option5.to_string()),
        ("SETTING_BACK".to_string(), text.key.setting_back.to_string()),
        ("SETTING_LANGUAGE".to_string(), text.setting.language.to_string()),
        ("SETTING_KEYBIND".to_string(), text.setting.keybind.to_string()),
        ("SETTING_MODS".to_string(), text.setting.mods.to_string()),
        ("SETTING_MEMORY".to_string(), text.setting.memory.to_string()),
        ("SETTING_SECURITY".to_string(), text.setting.security.to_string()),
    ]
}

fn pairs_to_table(lua: &Lua, pairs: &[(String, String)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (key, value) in pairs {
        table.set(key.as_str(), value.as_str())?;
    }
    Ok(table)
}
