//! 直用式辅助脚本加载 API 公开

use mlua::{Lua, Value, Variadic};

use super::module_loading_support::function_path;
use super::module_loading_support::function_runner;
use super::scope::ApiScope;
use super::validation::argument;
use crate::host_engine::boot::preload::lua_runtime::HostLuaBridge;

/// 安装辅助脚本加载 API。
pub fn install(lua: &Lua, api_scope: ApiScope, host_bridge: HostLuaBridge) -> mlua::Result<()> {
    if !api_scope.allows_module_loading() {
        return Ok(());
    }

    let globals = lua.globals();
    install_load_function(lua, &globals, host_bridge)?;

    Ok(())
}

fn install_load_function(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "load_function",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let logical_path = argument::expect_string_arg(&args, 0)?;
            let runtime_context = host_bridge.runtime_context();
            let package_root = runtime_context
                .current_game
                .as_ref()
                .map(|game_module| game_module.root_dir.as_path())
                .ok_or_else(|| mlua::Error::external("current package is unavailable"))?;
            let script_path =
                function_path::resolve_function_path(package_root, logical_path.as_str())?;
            if !script_path.is_file() {
                return Err(mlua::Error::external(format!(
                    "helper script not found: {}",
                    script_path.display()
                )));
            }
            function_runner::run_function_script(lua, &script_path)
        })?,
    )
}
