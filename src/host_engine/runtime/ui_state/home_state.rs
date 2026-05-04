//! UI Home 状态聚合

use crate::host_engine::boot::i18n;
use crate::host_engine::constant::HOST_VERSION;

use super::lua_state::HomeLuaState;
use super::root_state::{HomeConfirmAction, HomeRootState};

/// Home 页面宿主与 Lua 双层状态。
#[derive(Clone, Debug)]
pub struct HomeUiState {
    pub root_state: HomeRootState,
    pub lua_state: HomeLuaState,
}

impl HomeUiState {
    /// 创建初始 Home 状态。
    pub fn new() -> Self {
        let root_state = HomeRootState {
            language: home_language_pairs(),
            select: 1,
            version: HOST_VERSION.to_string(),
            continue_items: Vec::new(),
        };
        let lua_state = HomeLuaState::new(root_state.select);
        Self {
            root_state,
            lua_state,
        }
    }

    /// 进入 Home 页面时重置 Lua state，并从 root_state 同步 select。
    pub fn reset_lua_state(&mut self) {
        self.root_state.normalize_select();
        self.lua_state = HomeLuaState::new(self.root_state.select);
    }

    /// 应用 Lua 返回状态。
    pub fn apply_lua_state(&mut self, lua_state: HomeLuaState) -> Option<HomeConfirmAction> {
        self.lua_state = lua_state;
        self.root_state.select = self.lua_state.select;
        self.root_state.normalize_select();

        if self.lua_state.confirm {
            self.lua_state.confirm = false;
            return Some(self.root_state.confirm_action());
        }

        None
    }
}

fn home_language_pairs() -> Vec<(String, String)> {
    let text = i18n::text();
    vec![
        ("HOME_PREV_OPTION".to_string(), text.key.home_prev_option.to_string()),
        ("HOME_NEXT_OPTION".to_string(), text.key.home_next_option.to_string()),
        ("HOME_SELECT".to_string(), text.key.home_select.to_string()),
        ("HOME_CONFIRM".to_string(), text.key.home_confirm.to_string()),
        ("HOME_OPTION1".to_string(), text.key.home_option1.to_string()),
        ("HOME_OPTION2".to_string(), text.key.home_option2.to_string()),
        ("HOME_OPTION3".to_string(), text.key.home_option3.to_string()),
        ("HOME_OPTION4".to_string(), text.key.home_option4.to_string()),
        ("HOME_OPTION5".to_string(), text.key.home_option5.to_string()),
        ("HOME_PLAY".to_string(), text.home.play.to_string()),
        ("HOME_CONTINUE".to_string(), text.home.continue_game.to_string()),
        ("HOME_SETTINGS".to_string(), text.home.settings.to_string()),
        ("HOME_ABOUT".to_string(), text.home.about.to_string()),
        ("HOME_QUIT".to_string(), text.home.quit.to_string()),
    ]
}
