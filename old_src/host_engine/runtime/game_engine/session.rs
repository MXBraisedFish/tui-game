//! 单局游戏运行会话

use mlua::RegistryKey;

use crate::LuaRuntimeState;
use crate::host_engine::boot::preload::game_modules::GameModule;
use crate::host_engine::boot::preload::lua_runtime::api::{LuaEvent, callback_api};

/// 当前运行中的游戏会话。
pub(crate) struct GameSession {
    game_module: GameModule,
    state_key: RegistryKey,
}

impl GameSession {
    /// 创建游戏会话。
    pub(crate) fn new(game_module: GameModule, state_key: RegistryKey) -> Self {
        Self {
            game_module,
            state_key,
        }
    }

    /// 游戏目标 FPS。
    pub(crate) fn target_fps(&self) -> u64 {
        self.game_module.game.runtime.target_fps as u64
    }

    /// 游戏无操作低资源模式秒数。
    pub(crate) fn afk_time_secs(&self) -> u64 {
        self.game_module.game.runtime.afk_time
    }

    /// 当前游戏 UID。
    pub(crate) fn uid(&self) -> &str {
        self.game_module.uid.as_str()
    }

    /// 根据物理键查询动作。
    pub(crate) fn action_for_key(&self, keybinds: &serde_json::Value, key: &str) -> Option<String> {
        super::action_map::action_for_key(&self.game_module, keybinds, key)
    }

    /// 传递事件并更新游戏状态。
    pub(crate) fn handle_event(
        &mut self,
        lua_runtime: &LuaRuntimeState,
        event: LuaEvent,
    ) -> mlua::Result<()> {
        let lua = &lua_runtime.lua_runtime_environment.lua;
        let new_state_key = callback_api::call_handle_event(lua, &self.state_key, event)?;
        self.state_key = new_state_key;
        Ok(())
    }

    /// 绘制当前游戏画面。
    pub(crate) fn render(&self, lua_runtime: &LuaRuntimeState) -> mlua::Result<()> {
        let lua = &lua_runtime.lua_runtime_environment.lua;
        callback_api::call_render(lua, &self.state_key)
    }

    /// 调用游戏退出回调。
    pub(crate) fn exit_game(&mut self, lua_runtime: &LuaRuntimeState) -> mlua::Result<()> {
        let lua = &lua_runtime.lua_runtime_environment.lua;
        let new_state_key = callback_api::call_exit_game(lua, &self.state_key)?;
        self.state_key = new_state_key;
        Ok(())
    }

    /// 调用最佳记录保存回调。
    pub(crate) fn save_best_score(&self, lua_runtime: &LuaRuntimeState) -> mlua::Result<String> {
        let lua = &lua_runtime.lua_runtime_environment.lua;
        callback_api::call_save_best_score(lua, &self.state_key)
    }
}
