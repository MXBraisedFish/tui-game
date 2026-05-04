//! 直用式数据读取 API 公开

use mlua::{Lua, Value, Variadic};

use super::file_reading_support::asset_path;
use super::file_reading_support::file_reader;
use super::file_reading_support::lua_value;
use super::file_reading_support::structured_parser::{self, StructuredFormat};
use super::file_reading_support::translation_reader::{self, TranslationResult};
use super::scope::ApiScope;
use super::validation::argument;
use crate::host_engine::boot::preload::lua_runtime::HostLuaBridge;

/// 安装数据读取 API。
pub fn install(lua: &Lua, api_scope: ApiScope, host_bridge: HostLuaBridge) -> mlua::Result<()> {
    if !api_scope.allows_file_reading() {
        return Ok(());
    }

    let globals = lua.globals();
    install_translate(lua, &globals, host_bridge.clone())?;
    install_read_text(lua, &globals, host_bridge.clone())?;
    install_read_structured(
        lua,
        &globals,
        host_bridge.clone(),
        "read_json",
        StructuredFormat::Json,
    )?;
    install_read_structured(
        lua,
        &globals,
        host_bridge.clone(),
        "read_xml",
        StructuredFormat::Xml,
    )?;
    install_read_structured(
        lua,
        &globals,
        host_bridge.clone(),
        "read_yaml",
        StructuredFormat::Yaml,
    )?;
    install_read_structured(
        lua,
        &globals,
        host_bridge.clone(),
        "read_toml",
        StructuredFormat::Toml,
    )?;
    install_read_structured(
        lua,
        &globals,
        host_bridge.clone(),
        "read_csv",
        StructuredFormat::Csv,
    )?;
    install_read_raw_string(lua, &globals, host_bridge.clone(), "read_json_string")?;
    install_read_raw_string(lua, &globals, host_bridge.clone(), "read_xml_string")?;
    install_read_raw_string(lua, &globals, host_bridge.clone(), "read_yaml_string")?;
    install_read_raw_string(lua, &globals, host_bridge.clone(), "read_toml_string")?;
    install_read_raw_string(lua, &globals, host_bridge, "read_csv_string")?;

    Ok(())
}

fn install_translate(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "translate",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let key = argument::expect_string_arg(&args, 0)?;
            let runtime_context = host_bridge.runtime_context();
            let package_root = runtime_context
                .current_game
                .as_ref()
                .map(|game_module| game_module.root_dir.as_path())
                .ok_or_else(|| mlua::Error::external("current package is unavailable"))?;
            match translation_reader::read_translation(
                package_root,
                runtime_context.language_code.as_str(),
                key.as_str(),
            )? {
                TranslationResult::Found(value) => Ok(Value::String(lua.create_string(&value)?)),
                TranslationResult::MissingInCurrentLanguage
                | TranslationResult::MissingInFallback => Ok(Value::String(
                    lua.create_string(format!("[missing-i18n-key: {key}]").as_str())?,
                )),
            }
        })?,
    )
}

fn install_read_text(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "read_text",
        lua.create_function(move |lua, args: Variadic<Value>| {
            let text = read_asset_text(&host_bridge, &args)?;
            Ok(Value::String(lua.create_string(&text)?))
        })?,
    )
}

fn install_read_structured(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
    function_name: &'static str,
    file_format: StructuredFormat,
) -> mlua::Result<()> {
    globals.set(
        function_name,
        lua.create_function(move |lua, args: Variadic<Value>| {
            let text = read_asset_text(&host_bridge, &args)?;
            let value = structured_parser::parse_structured_text(text.as_str(), file_format)?;
            lua_value::json_to_lua_value(lua, &value)
        })?,
    )
}

fn install_read_raw_string(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
    function_name: &'static str,
) -> mlua::Result<()> {
    globals.set(
        function_name,
        lua.create_function(move |lua, args: Variadic<Value>| {
            let text = read_asset_text(&host_bridge, &args)?;
            Ok(Value::String(lua.create_string(&text)?))
        })?,
    )
}

fn read_asset_text(host_bridge: &HostLuaBridge, args: &Variadic<Value>) -> mlua::Result<String> {
    argument::expect_exact_arg_count(args, 1)?;
    let logical_path = argument::expect_string_arg(args, 0)?;
    let runtime_context = host_bridge.runtime_context();
    let package_root = runtime_context
        .current_game
        .as_ref()
        .map(|game_module| game_module.root_dir.as_path())
        .ok_or_else(|| mlua::Error::external("current package is unavailable"))?;
    let resolved_path = asset_path::resolve_asset_path(package_root, logical_path.as_str())?;
    file_reader::ensure_file_exists(&resolved_path)?;
    file_reader::read_text(&resolved_path)
}
