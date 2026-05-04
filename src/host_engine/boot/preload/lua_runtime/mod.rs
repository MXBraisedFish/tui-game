//! Lua 虚拟机与沙箱环境预加载入口

pub(crate) mod api;
mod environment;
mod host_bridge;
mod sandbox;

pub use environment::LuaRuntimeEnvironment;
pub use host_bridge::{
    HostLuaBridge, HostLuaMessage, LaunchMode, LuaRuntimeConsumer, LuaRuntimeContext,
};

use crate::LoadedResources;

/// 准备 Lua 虚拟机和沙箱环境。
///
/// 当前阶段只做：
/// - 创建 Lua VM。
/// - 建立宿主与 Lua 通信桥占位。
/// - 安装沙箱限制。
///
/// TODO: 后续在这里按模块公开宿主自定义 API。
/// TODO: 后续在这里加载官方 UI Lua 脚本和游戏脚本运行上下文。
pub(crate) fn load(
    loaded_resources: &LoadedResources,
) -> Result<LuaRuntimeEnvironment, Box<dyn std::error::Error>> {
    let host_bridge = HostLuaBridge::new();
    host_bridge.set_runtime_context(LuaRuntimeContext {
        consumer: LuaRuntimeConsumer::GamePackage,
        current_game: loaded_resources.game_module_registry.games.first().cloned(),
        language_code: loaded_resources.persistent_data.language_code.clone(),
        keybinds: loaded_resources.persistent_data.keybinds.clone(),
        best_scores: loaded_resources.persistent_data.best_scores.clone(),
        mod_state: loaded_resources.persistent_data.mod_state.clone(),
        launch_mode: LaunchMode::New,
        terminal_size: loaded_resources.initialized_environment.terminal_size,
    });
    let lua_runtime_environment = environment::create_lua_runtime_environment(host_bridge)?;
    Ok(lua_runtime_environment)
}
