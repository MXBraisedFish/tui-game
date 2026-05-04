//! 直用式表处理工具 API 公开

use mlua::{Lua, Value, Variadic};

use super::scope::ApiScope;
use super::table_utilities_support::deep_copy;
use super::table_utilities_support::lua_table_converter;
use super::table_utilities_support::table_serializer::{self, TableFormat};
use super::validation::argument;

/// 安装表处理工具 API。
pub fn install(lua: &Lua, api_scope: ApiScope) -> mlua::Result<()> {
    if !api_scope.allows_table_utilities() {
        return Ok(());
    }

    let globals = lua.globals();
    install_table_to_string(lua, &globals, "table_to_json", TableFormat::Json)?;
    install_table_to_string(lua, &globals, "table_to_yaml", TableFormat::Yaml)?;
    install_table_to_string(lua, &globals, "table_to_toml", TableFormat::Toml)?;
    install_table_to_string(lua, &globals, "table_to_csv", TableFormat::Csv)?;
    install_table_to_string(lua, &globals, "table_to_xml", TableFormat::Xml)?;
    install_deep_copy(lua, &globals)?;

    Ok(())
}

fn install_table_to_string(
    lua: &Lua,
    globals: &mlua::Table,
    function_name: &'static str,
    table_format: TableFormat,
) -> mlua::Result<()> {
    globals.set(
        function_name,
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let table = expect_table_arg(&args, 0)?;
            let value = lua_table_converter::lua_table_to_json(&table)?;
            let serialized = table_serializer::serialize_table(&value, table_format)?;
            Ok(Value::String(lua.create_string(serialized.as_str())?))
        })?,
    )
}

fn install_deep_copy(lua: &Lua, globals: &mlua::Table) -> mlua::Result<()> {
    globals.set(
        "deep_copy",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let table = expect_table_arg(&args, 0)?;
            Ok(Value::Table(deep_copy::deep_copy_table(lua, &table)?))
        })?,
    )
}

fn expect_table_arg(args: &Variadic<Value>, index: usize) -> mlua::Result<mlua::Table> {
    match args.get(index) {
        Some(Value::Table(table)) => Ok(table.clone()),
        Some(value) => Err(mlua::Error::external(format!(
            "argument type mismatch: expected table, got {}",
            lua_type_name(value)
        ))),
        None => Err(mlua::Error::external("argument missing")),
    }
}

fn lua_type_name(value: &Value) -> &'static str {
    match value {
        Value::Nil => "nil",
        Value::Boolean(_) => "boolean",
        Value::LightUserData(_) => "light_userdata",
        Value::Integer(_) => "integer",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Table(_) => "table",
        Value::Function(_) => "function",
        Value::Thread(_) => "thread",
        Value::UserData(_) => "userdata",
        Value::Error(_) => "error",
        Value::Other(_) => "other",
    }
}
