//! 声明式 callback API 公开

use mlua::{Function, Lua, RegistryKey, Value};

use super::scope::ApiScope;
use super::validation::callback_contract;
use super::value::event_value::LuaEvent;

/// 安装当前作用域允许使用的声明式 API 调用器。
///
/// 注意：声明式 API 是 Lua 脚本重写的函数，宿主不会向 Lua 注入这些函数；
/// 此文件负责公开 Rust 侧调用这些声明式函数的统一入口。
pub fn install(_lua: &Lua, _api_scope: ApiScope) -> mlua::Result<()> {
    // 声明式 API 由脚本自行定义，这里不向 globals 注入同名函数。
    // TODO: 后续如果需要，可在这里安装 callback 元数据或开发期检查工具。
    Ok(())
}

/// 校验当前 Lua 环境是否实现了作用域要求的声明式 API。
pub fn validate_required_callbacks(lua: &Lua, api_scope: ApiScope) -> mlua::Result<()> {
    if api_scope.allows_ui_callbacks() {
        callback_contract::require_function(lua, "handle_event")?;
        callback_contract::require_function(lua, "render")?;
    }

    if api_scope.allows_game_callbacks() {
        callback_contract::require_function(lua, "init_game")?;
        callback_contract::require_function(lua, "exit_game")?;
        // TODO: 根据 game.json 的 best_none 决定是否要求 save_best_score。
        // TODO: 根据 game.json 的 save 决定是否要求 save_game。
    }

    Ok(())
}

/// 调用游戏初始化函数 init_game(state)。
pub fn call_init_game(lua: &Lua, incoming_state: Value) -> mlua::Result<RegistryKey> {
    let init_game: Function = lua.globals().get("init_game")?;
    let state = init_game.call::<Value>(incoming_state)?;
    callback_contract::ensure_returned_value(&state)?;
    lua.create_registry_value(state)
}

/// 调用事件处理函数 handle_event(state, event)。
pub fn call_handle_event(
    lua: &Lua,
    state_key: &RegistryKey,
    event: LuaEvent,
) -> mlua::Result<RegistryKey> {
    let handle_event: Function = lua.globals().get("handle_event")?;
    let state = lua.registry_value::<Value>(state_key)?;
    let event_table = event.into_lua_table(lua)?;
    let new_state = handle_event.call::<Value>((state, event_table))?;
    callback_contract::ensure_returned_value(&new_state)?;
    lua.create_registry_value(new_state)
}

/// 调用渲染函数 render(state)。
pub fn call_render(lua: &Lua, state_key: &RegistryKey) -> mlua::Result<()> {
    let render: Function = lua.globals().get("render")?;
    let state = lua.registry_value::<Value>(state_key)?;
    render.call::<()>(state)
}

/// 调用退出函数 exit_game(state)。
pub fn call_exit_game(lua: &Lua, state_key: &RegistryKey) -> mlua::Result<RegistryKey> {
    let exit_game: Function = lua.globals().get("exit_game")?;
    let state = lua.registry_value::<Value>(state_key)?;
    let new_state = exit_game.call::<Value>(state)?;
    callback_contract::ensure_returned_value(&new_state)?;
    lua.create_registry_value(new_state)
}

/// 调用最佳记录保存函数 save_best_score(state)。
pub fn call_save_best_score(lua: &Lua, state_key: &RegistryKey) -> mlua::Result<Value> {
    let save_best_score: Function = lua.globals().get("save_best_score")?;
    let state = lua.registry_value::<Value>(state_key)?;
    let best_score = save_best_score.call::<Value>(state)?;
    callback_contract::ensure_returned_value(&best_score)?;
    Ok(best_score)
}

/// 调用游戏存档函数 save_game(state)。
pub fn call_save_game(lua: &Lua, state_key: &RegistryKey) -> mlua::Result<Value> {
    let save_game: Function = lua.globals().get("save_game")?;
    let state = lua.registry_value::<Value>(state_key)?;
    let save_state = save_game.call::<Value>(state)?;
    callback_contract::ensure_returned_value(&save_state)?;
    Ok(save_state)
}
