//! UI Security 状态聚合

use mlua::{Lua, Table, Value};

use crate::host_engine::boot::i18n;
use crate::host_engine::boot::preload::persistent_data::security_profile::SecurityProfile;

pub const SECURITY_OPTION_COUNT: i64 = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecurityConfirmAction {
    ToggleDefaultSafeMode,
    ToggleDefaultModGame,
    ToggleDefaultModScreensaver,
    ToggleDefaultModBoss,
    ResetSafeMode,
    ResetModGame,
    ResetModScreensaver,
    ResetModBoss,
}

#[derive(Clone, Debug)]
pub struct SecurityUiState {
    pub root_state: SecurityRootState,
    pub lua_state: SecurityLuaState,
}
impl SecurityUiState {
    pub fn new(profile: SecurityProfile) -> Self {
        let root_state = SecurityRootState::new(1, profile);
        let lua_state = SecurityLuaState::new(root_state.select);
        Self {
            root_state,
            lua_state,
        }
    }
    pub fn reset_lua_state(&mut self) {
        self.root_state.normalize_select();
        self.lua_state = SecurityLuaState::new(self.root_state.select);
    }
    pub fn refresh_language(&mut self) {
        self.root_state.language = security_language_pairs();
    }
    pub fn profile(&self) -> SecurityProfile {
        SecurityProfile {
            default_safe_mode: self.root_state.default_safe_mode,
            default_mod_game_enabled: self.root_state.default_mod_game_enabled,
            default_mod_screensaver_enabled: self.root_state.default_mod_screensaver_enabled,
            default_mod_boss_enabled: self.root_state.default_mod_boss_enabled,
        }
    }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecurityLuaAction {
    None,
    Back,
    Confirm(SecurityConfirmAction),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityLuaState {
    pub select: i64,
    pub confirm: bool,
    pub back: bool,
}
impl SecurityLuaState {
    pub fn new(select: i64) -> Self {
        Self {
            select: normalize_security_select(select),
            confirm: false,
            back: false,
        }
    }
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let t = lua.create_table()?;
        t.set("select", normalize_security_select(self.select))?;
        t.set("confirm", false)?;
        t.set("back", false)?;
        Ok(t)
    }
    pub fn from_lua_value(value: Value) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(t) => t,
            _ => {
                return Err(mlua::Error::external(
                    "security lua state must be returned as table",
                ));
            }
        };
        Ok(Self {
            select: normalize_security_select(table.get::<Option<i64>>("select")?.unwrap_or(1)),
            confirm: table.get::<Option<bool>>("confirm")?.unwrap_or(false),
            back: table.get::<Option<bool>>("back")?.unwrap_or(false),
        })
    }
}

#[derive(Clone, Debug)]
pub struct SecurityRootState {
    pub language: Vec<(String, String)>,
    pub select: i64,
    pub default_safe_mode: bool,
    pub default_mod_game_enabled: bool,
    pub default_mod_screensaver_enabled: bool,
    pub default_mod_boss_enabled: bool,
    pub reset_message: Option<String>,
}
impl SecurityRootState {
    pub fn new(select: i64, profile: SecurityProfile) -> Self {
        Self {
            language: security_language_pairs(),
            select: normalize_security_select(select),
            default_safe_mode: profile.default_safe_mode,
            default_mod_game_enabled: profile.default_mod_game_enabled,
            default_mod_screensaver_enabled: profile.default_mod_screensaver_enabled,
            default_mod_boss_enabled: profile.default_mod_boss_enabled,
            reset_message: None,
        }
    }
    pub fn normalize_select(&mut self) {
        self.select = normalize_security_select(self.select);
    }
    pub fn confirm_action(&self) -> SecurityConfirmAction {
        match normalize_security_select(self.select) {
            1 => SecurityConfirmAction::ToggleDefaultSafeMode,
            2 => SecurityConfirmAction::ToggleDefaultModGame,
            3 => SecurityConfirmAction::ToggleDefaultModScreensaver,
            4 => SecurityConfirmAction::ToggleDefaultModBoss,
            5 => SecurityConfirmAction::ResetSafeMode,
            6 => SecurityConfirmAction::ResetModGame,
            7 => SecurityConfirmAction::ResetModScreensaver,
            8 => SecurityConfirmAction::ResetModBoss,
            _ => SecurityConfirmAction::ToggleDefaultSafeMode,
        }
    }
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let t = lua.create_table()?;
        t.set("language", pairs_to_table(lua, &self.language)?)?;
        t.set("select", normalize_security_select(self.select))?;
        t.set("default_safe_mode", self.default_safe_mode)?;
        t.set("default_mod_game_enabled", self.default_mod_game_enabled)?;
        t.set("default_mod_screensaver_enabled", self.default_mod_screensaver_enabled)?;
        t.set("default_mod_boss_enabled", self.default_mod_boss_enabled)?;
        t.set(
            "reset_message",
            self.reset_message.clone().unwrap_or_default(),
        )?;
        Ok(t)
    }
}

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
        ("SECURITY_PREV_OPTION".into(), text.key.security_prev_option),
        ("SECURITY_NEXT_OPTION".into(), text.key.security_next_option),
        ("SECURITY_SELECT".into(), text.key.security_select),
        (
            "SECURITY_CLOSE_PERMANENT".into(),
            text.key.security_close_permanent,
        ),
        ("SECURITY_BACK".into(), text.key.security_back),
        (
            "SECURITY_TOGGLE_CONFIRM".into(),
            text.key.security_toggle_confirm,
        ),
        ("SECURITY_TOGGLE".into(), text.key.security_toggle),
        ("SECURITY_CONFIRM".into(), text.key.security_confirm),
        ("SECURITY_OPTION1".into(), text.key.security_option1),
        ("SECURITY_OPTION2".into(), text.key.security_option2),
        ("SECURITY_OPTION3".into(), text.key.security_option3),
        ("SECURITY_OPTION4".into(), text.key.security_option4),
        ("SECURITY_OPTION5".into(), text.key.security_option5),
        ("SECURITY_OPTION6".into(), text.key.security_option6),
        ("SECURITY_OPTION7".into(), text.key.security_option7),
        ("SECURITY_OPTION8".into(), text.key.security_option8),
        ("SECURITY_TITLE".into(), text.security.title),
        ("SECURITY_TOGGLE_MOD_ON".into(), text.security.toggle_mod_on),
        (
            "SECURITY_TOGGLE_MOD_OFF".into(),
            text.security.toggle_mod_off,
        ),
        (
            "SECURITY_TOGGLE_SAFE_MODE_ON".into(),
            text.security.toggle_safe_mode_on,
        ),
        (
            "SECURITY_TOGGLE_SAFE_MODE_OFF_PERMANENT".into(),
            text.security.toggle_safe_mode_off_permanent,
        ),
        (
            "SECURITY_DEFAULT_SAFE_MODE".into(),
            text.security.default_safe_mode,
        ),
        (
            "SECURITY_DEFAULT_MOD_GAME".into(),
            text.security.default_mod_game,
        ),
        (
            "SECURITY_DEFAULT_MOD_SCREENSAVER".into(),
            text.security.default_mod_screensaver,
        ),
        (
            "SECURITY_DEFAULT_MOD_BOSS".into(),
            text.security.default_mod_boss,
        ),
        (
            "SECURITY_RESET_SAFE_MODE".into(),
            text.security.reset_safe_mode,
        ),
        (
            "SECURITY_RESET_MOD_GAME".into(),
            text.security.reset_mod_game,
        ),
        (
            "SECURITY_RESET_MOD_SCREENSAVER".into(),
            text.security.reset_mod_screensaver,
        ),
        (
            "SECURITY_RESET_MOD_BOSS".into(),
            text.security.reset_mod_boss,
        ),
        ("SECURITY_RESET_SUCCESS".into(), text.security.reset_success),
        ("SECURITY_RESET_FAILED".into(), text.security.reset_failed),
    ]
}
fn pairs_to_table(lua: &Lua, pairs: &[(String, String)]) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    for (k, v) in pairs {
        t.set(k.as_str(), v.as_str())?;
    }
    Ok(t)
}
