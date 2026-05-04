//! 高风险写入权限判断

use serde_json::Value;

use crate::host_engine::boot::preload::game_modules::GameModuleSource;
use crate::host_engine::boot::preload::lua_runtime::HostLuaBridge;

/// 判断当前游戏是否允许直写资源文件。
pub fn can_write_assets(host_bridge: &HostLuaBridge) -> bool {
    let runtime_context = host_bridge.runtime_context();
    let Some(game_module) = runtime_context.current_game.as_ref() else {
        return false;
    };

    if !game_module.game.write {
        return false;
    }

    match game_module.source {
        GameModuleSource::Office => true,
        GameModuleSource::Mod => {
            is_mod_fully_trusted(&runtime_context.mod_state, game_module.uid.as_str())
        }
    }
}

fn is_mod_fully_trusted(mod_state: &Value, game_uid: &str) -> bool {
    mod_state
        .get(game_uid)
        .and_then(|state| state.get("safe_mode"))
        .and_then(Value::as_bool)
        .map(|safe_mode| !safe_mode)
        .unwrap_or(false)
}
