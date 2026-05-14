//! 游戏入口脚本加载

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use mlua::Value;

use crate::LuaRuntimeState;
use crate::host_engine::boot::preload::game_modules::GameModule;
use crate::host_engine::boot::preload::lua_runtime::api::{self, ApiScope, callback_api};
use crate::host_engine::boot::preload::lua_runtime::{
    LaunchMode, LuaRuntimeConsumer, LuaRuntimeContext,
};
use crate::host_engine::runtime::game_engine::session::GameSession;

type ScriptLoaderResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 加载游戏入口脚本并调用 init_game。
pub fn load_new_game(
    lua_runtime: &LuaRuntimeState,
    game_module: GameModule,
) -> ScriptLoaderResult<GameSession> {
    let lua = &lua_runtime.lua_runtime_environment.lua;
    let host_bridge = &lua_runtime.lua_runtime_environment.host_bridge;
    let current_context = host_bridge.runtime_context();
    let script_root = game_module.root_dir.join("scripts");
    let script_path = resolve_script_path(&script_root, game_module.game.entry.as_str())?;

    host_bridge.set_runtime_context(LuaRuntimeContext {
        consumer: LuaRuntimeConsumer::GamePackage,
        current_game: Some(game_module.clone()),
        current_overlay: None,
        current_ui_actions: serde_json::Value::Null,
        current_script_root: Some(script_root),
        language_code: current_context.language_code,
        keybinds: current_context.keybinds,
        best_scores: current_context.best_scores,
        mod_state: current_context.mod_state,
        saver_state: current_context.saver_state,
        boss_state: current_context.boss_state,
        launch_mode: LaunchMode::New,
        terminal_size: current_context.terminal_size,
        is_focused: current_context.is_focused,
    });
    api::install_runtime_apis(lua, ApiScope::game_package(), host_bridge.clone())?;

    let source = fs::read_to_string(&script_path)
        .map(|text| text.trim_start_matches('\u{feff}').to_string())?;
    lua.load(source.as_str())
        .set_name(script_path.to_string_lossy().as_ref())
        .exec()?;
    callback_api::validate_required_callbacks(lua, ApiScope::game_package())?;

    let state_key = callback_api::call_init_game(lua, Value::Nil)?;
    Ok(GameSession::new(game_module, state_key))
}

fn resolve_script_path(script_root: &Path, logical_path: &str) -> ScriptLoaderResult<PathBuf> {
    let trimmed_path = logical_path.trim();
    if trimmed_path.is_empty() || Path::new(trimmed_path).is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid game script path: {trimmed_path}"),
        )
        .into());
    }

    let mut clean_path = PathBuf::new();
    for component in PathBuf::from(trimmed_path).components() {
        match component {
            Component::Normal(part) => clean_path.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("invalid game script path: {trimmed_path}"),
                )
                .into());
            }
        }
    }

    Ok(script_root.join(clean_path))
}
