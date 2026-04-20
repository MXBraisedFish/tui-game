pub mod callback_api;
pub mod common;
pub mod direct_debug_api;
pub mod direct_drawing_api;
pub mod direct_file_reading_api;
pub mod direct_file_writing_api;
pub mod direct_layout_api;
pub mod direct_measurement_api;
pub mod direct_module_loading_api;
pub mod direct_random_api;
pub mod direct_system_request_api;
pub mod direct_table_utilities_api;
pub mod direct_timer_api;

use mlua::Lua;

use crate::lua::engine::RuntimeBridges;

pub(crate) fn install_runtime_apis(lua: &Lua, bridges: &RuntimeBridges) -> mlua::Result<()> {
    direct_drawing_api::install(lua, bridges.clone())?;
    direct_file_reading_api::install(lua, bridges.clone())?;
    direct_file_writing_api::install(lua, bridges.clone())?;
    direct_layout_api::install(lua)?;
    direct_measurement_api::install(lua, bridges.clone())?;
    direct_module_loading_api::install(lua, bridges.clone())?;
    direct_random_api::install(lua, bridges.clone())?;
    direct_debug_api::install(lua, bridges.clone())?;
    direct_system_request_api::install(lua, bridges.clone())?;
    direct_table_utilities_api::install(lua, bridges.clone())?;
    direct_timer_api::install(lua, bridges.clone())?;
    Ok(())
}
