//! Lua 自定义 API 公开入口
#![allow(dead_code)]

pub mod callback_api;
mod debug_support;
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
pub(crate) mod drawing_support;
mod file_reading_support;
mod file_writing_support;
mod layout_support;
mod measurement_support;
mod module_loading_support;
pub(crate) mod random_support;
mod scope;
mod table_utilities_support;
pub(crate) mod timer_support;
mod validation;
mod value;

pub use scope::ApiScope;

use mlua::Lua;

use super::HostLuaBridge;

/// 安装指定作用域允许使用的宿主 API。
pub fn install_runtime_apis(
    lua: &Lua,
    api_scope: ApiScope,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    callback_api::install(lua, api_scope)?;
    direct_drawing_api::install(lua, api_scope, host_bridge.clone())?;
    direct_measurement_api::install(lua, api_scope, host_bridge.clone())?;
    direct_layout_api::install(lua, api_scope, host_bridge.clone())?;
    direct_file_reading_api::install(lua, api_scope, host_bridge.clone())?;
    direct_file_writing_api::install(lua, api_scope, host_bridge.clone())?;
    direct_table_utilities_api::install(lua, api_scope)?;
    direct_module_loading_api::install(lua, api_scope, host_bridge.clone())?;
    direct_timer_api::install(lua, api_scope, host_bridge.clone())?;
    direct_random_api::install(lua, api_scope, host_bridge.clone())?;
    direct_debug_api::install(lua, api_scope, host_bridge.clone())?;
    direct_system_request_api::install(lua, api_scope, host_bridge)?;
    Ok(())
}
