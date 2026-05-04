//! Lua 运行时环境

use mlua::Lua;

use super::api::{self, ApiScope};
use super::host_bridge::HostLuaBridge;
use super::sandbox;

/// Lua 虚拟机与宿主通信环境。
pub struct LuaRuntimeEnvironment {
    pub lua: Lua,
    pub host_bridge: HostLuaBridge,
    pub is_sandbox_installed: bool,
}

impl LuaRuntimeEnvironment {
    /// 当前 Lua 沙箱是否已安装。
    pub fn is_sandbox_installed(&self) -> bool {
        self.is_sandbox_installed
    }
}

/// 创建 Lua VM 并安装沙箱。
pub fn create_lua_runtime_environment(
    host_bridge: HostLuaBridge,
) -> Result<LuaRuntimeEnvironment, Box<dyn std::error::Error>> {
    let lua = Lua::new();
    sandbox::install_sandbox(&lua)?;
    api::install_runtime_apis(&lua, ApiScope::game_package(), host_bridge.clone())?;

    Ok(LuaRuntimeEnvironment {
        lua,
        host_bridge,
        is_sandbox_installed: true,
    })
}
