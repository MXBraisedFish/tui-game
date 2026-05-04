//! 直用式数据写入 API 公开

use mlua::{Lua, Value, Variadic};

use super::file_reading_support::asset_path;
use super::file_writing_support::audit_log::{self, WriteAuditStatus};
use super::file_writing_support::write_permission;
use super::file_writing_support::writer;
use super::scope::ApiScope;
use super::validation::argument;
use crate::host_engine::boot::preload::lua_runtime::HostLuaBridge;

/// 安装数据写入 API。
pub fn install(lua: &Lua, api_scope: ApiScope, host_bridge: HostLuaBridge) -> mlua::Result<()> {
    if !api_scope.allows_file_writing() {
        return Ok(());
    }

    let globals = lua.globals();
    install_write_api(lua, &globals, host_bridge.clone(), "write_text")?;
    install_write_api(lua, &globals, host_bridge.clone(), "write_json")?;
    install_write_api(lua, &globals, host_bridge.clone(), "write_xml")?;
    install_write_api(lua, &globals, host_bridge.clone(), "write_yaml")?;
    install_write_api(lua, &globals, host_bridge.clone(), "write_toml")?;
    install_write_api(lua, &globals, host_bridge, "write_csv")?;

    Ok(())
}

fn install_write_api(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
    api_name: &'static str,
) -> mlua::Result<()> {
    globals.set(
        api_name,
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 2)?;
            let logical_path = argument::expect_string_arg(&args, 0)?;
            let content = argument::expect_string_arg(&args, 1)?;
            let runtime_context = host_bridge.runtime_context();
            let package_root = runtime_context
                .current_game
                .as_ref()
                .map(|game_module| game_module.root_dir.as_path())
                .ok_or_else(|| mlua::Error::external("current package is unavailable"))?;
            let resolved_path =
                asset_path::resolve_asset_path(package_root, logical_path.as_str())?;
            let can_write = write_permission::can_write_assets(&host_bridge);

            if !can_write {
                audit_log::append_write_request(
                    &host_bridge,
                    api_name,
                    &resolved_path,
                    WriteAuditStatus::Denied,
                )?;
                return Ok(false);
            }

            let succeeded = writer::write_text(&resolved_path, content.as_str());
            let audit_status = if succeeded {
                WriteAuditStatus::Allowed
            } else {
                WriteAuditStatus::Denied
            };
            audit_log::append_write_request(&host_bridge, api_name, &resolved_path, audit_status)?;
            Ok(succeeded)
        })?,
    )
}
