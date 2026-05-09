//! UI Security 状态聚合

use mlua::{Lua, Table, Value};

use crate::host_engine::boot::i18n;

/// Security 页面选项数量。
pub const SECURITY_OPTION_COUNT: i64 = 4;

/// Security 页面确认动作。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecurityConfirmAction {
    ToggleDefaultSafeMode,
    ToggleDefaultMod,
    ResetSafeMode,
    ResetMod,
}

/// Security 页面宿主与 Lua 双层状态。
#[derive(Clone, Debug)]
pub struct SecurityUiState {
    pub root_state: SecurityRootState,
    pub lua_state: SecurityLuaState,
}

impl SecurityUiState {
    /// 创建初始 Security 状态。
    pub fn new() -> Self {
        let root_state = SecurityRootState::new(1);
        let lua_state = SecurityLuaState::new(root_state.select);
        Self {
            root_state,
            lua_state,
        }
    }

    /// 进入 Security 页面时重置 Lua state，并从 root_state 同步 select。
    pub fn reset_lua_state(&mut self) {
        self.root_state.normalize_select();
        self.lua_state = SecurityLuaState::new(self.root_state.select);
    }

    /// 刷新 Security 页面语言文本。
    pub fn refresh_language(&mut self) {
        self.root_state.language = security_language_pairs();
    }

    /// 应用 Lua 返回状态。
    pub fn apply_lua_state(&mut self, lua_state: SecurityLuaState) -> SecurityLuaAction {
        self.lua_state = lua_state;
        self.root_state.select = self.lua_state.select;
        self.root_state.normalize_select();

        if self.lua_state.back {
            self.lua_state.back = false;
            return SecurityLuaAction::Back;
        }

        if self.lua_state.confirm {
            self.lua_state.confirm = false;
            return SecurityLuaAction::Confirm(self.root_state.confirm_action());
        }

        SecurityLuaAction::None
    }
}

/// Security Lua 返回动作。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecurityLuaAction {
    None,
    Back,
    Confirm(SecurityConfirmAction),
}

/// Security 页面 Lua 运行状态。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityLuaState {
    pub select: i64,
    pub confirm: bool,
    pub back: bool,
}

impl SecurityLuaState {
    /// 创建新的 Lua state，确认与返回状态总是重置为 false。
    pub fn new(select: i64) -> Self {
        Self {
            select: normalize_security_select(select),
            confirm: false,
            back: false,
        }
    }

    /// 转为 Lua 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("select", normalize_security_select(self.select))?;
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
                    "security lua state must be returned as table",
                ));
            }
        };
        let select = table.get::<Option<i64>>("select")?.unwrap_or(1);
        let confirm = table.get::<Option<bool>>("confirm")?.unwrap_or(false);
        let back = table.get::<Option<bool>>("back")?.unwrap_or(false);
        Ok(Self {
            select: normalize_security_select(select),
            confirm,
            back,
        })
    }
}

/// Security 页面宿主根状态。
#[derive(Clone, Debug)]
pub struct SecurityRootState {
    pub language: Vec<(String, String)>,
    pub select: i64,
    pub default_safe_mode: bool,
    pub default_mod_enabled: bool,
}

impl SecurityRootState {
    /// 创建新的 Security root state。
    pub fn new(select: i64) -> Self {
        Self {
            language: security_language_pairs(),
            select: normalize_security_select(select),
            default_safe_mode: true,
            default_mod_enabled: true,
        }
    }

    /// 规范化当前选项。
    pub fn normalize_select(&mut self) {
        self.select = normalize_security_select(self.select);
    }

    /// 选中项转为确认动作。
    pub fn confirm_action(&self) -> SecurityConfirmAction {
        match normalize_security_select(self.select) {
            1 => SecurityConfirmAction::ToggleDefaultSafeMode,
            2 => SecurityConfirmAction::ToggleDefaultMod,
            3 => SecurityConfirmAction::ResetSafeMode,
            4 => SecurityConfirmAction::ResetMod,
            _ => SecurityConfirmAction::ToggleDefaultSafeMode,
        }
    }

    /// 转为 Lua root_state 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("language", pairs_to_table(lua, &self.language)?)?;
        table.set("select", normalize_security_select(self.select))?;
        table.set("default_safe_mode", self.default_safe_mode)?;
        table.set("default_mod_enabled", self.default_mod_enabled)?;
        Ok(table)
    }
}

/// 将 Security 选项限制在 1-4。
pub fn normalize_security_select(select: i64) -> i64 {
    if select < 1 {
        SECURITY_OPTION_COUNT
    } else if select > SECURITY_OPTION_COUNT {
        1
    } else {
        select
    }
}

fn security_language_pairs() -> Vec<(String, String)> {
    let text = i18n::text();
    vec![
        (
            "SECURITY_PREV_OPTION".to_string(),
            text.key.security_prev_option,
        ),
        (
            "SECURITY_NEXT_OPTION".to_string(),
            text.key.security_next_option,
        ),
        ("SECURITY_SELECT".to_string(), text.key.security_select),
        (
            "SECURITY_CLOSE_PERMANENT".to_string(),
            text.key.security_close_permanent,
        ),
        ("SECURITY_BACK".to_string(), text.key.security_back),
        (
            "SECURITY_TOGGLE_CONFIRM".to_string(),
            text.key.security_toggle_confirm,
        ),
        ("SECURITY_TOGGLE".to_string(), text.key.security_toggle),
        ("SECURITY_CONFIRM".to_string(), text.key.security_confirm),
        ("SECURITY_OPTION1".to_string(), text.key.security_option1),
        ("SECURITY_OPTION2".to_string(), text.key.security_option2),
        ("SECURITY_OPTION3".to_string(), text.key.security_option3),
        ("SECURITY_OPTION4".to_string(), text.key.security_option4),
        ("SECURITY_TITLE".to_string(), text.security.title),
        (
            "SECURITY_TOGGLE_MOD_ON".to_string(),
            text.security.toggle_mod_on,
        ),
        (
            "SECURITY_TOGGLE_MOD_OFF".to_string(),
            text.security.toggle_mod_off,
        ),
        (
            "SECURITY_TOGGLE_SAFE_MODE_ON".to_string(),
            text.security.toggle_safe_mode_on,
        ),
        (
            "SECURITY_TOGGLE_SAFE_MODE_OFF_PERMANENT".to_string(),
            text.security.toggle_safe_mode_off_permanent,
        ),
        (
            "SECURITY_DEFAULT_SAFE_MODE".to_string(),
            text.security.default_safe_mode,
        ),
        (
            "SECURITY_DEFAULT_MOD".to_string(),
            text.security.default_mod,
        ),
        (
            "SECURITY_RESET_SAFE_MODE".to_string(),
            text.security.reset_safe_mode,
        ),
        ("SECURITY_RESET_MOD".to_string(), text.security.reset_mod),
    ]
}

fn pairs_to_table(lua: &Lua, pairs: &[(String, String)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (key, value) in pairs {
        table.set(key.as_str(), value.as_str())?;
    }
    Ok(table)
}
