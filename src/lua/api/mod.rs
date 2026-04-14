pub mod callback_api;
pub mod direct_debug_api;
pub mod direct_drawing_api;
pub mod direct_system_control_api;

use mlua::Lua;

use crate::lua::engine::RuntimeBridges;

pub(crate) fn install_runtime_apis(lua: &Lua, bridges: &RuntimeBridges) -> mlua::Result<()> {
    direct_drawing_api::install(lua, bridges.clone())?;
    direct_debug_api::install(lua, bridges.clone())?;
    direct_system_control_api::install(lua, bridges.clone())?;
    Ok(())
}
