use csv::WriterBuilder;
use mlua::{Lua, Table, Value, Variadic};
use serde_json::{Map, Number, Value as JsonValue};

use crate::lua::api::common;
use crate::lua::api::direct_debug_api;
use crate::lua::engine::RuntimeBridges;

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    {
        let bridges = bridges.clone();
        globals.set(
            "table_to_json",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let table = common::expect_table_arg(&args, 0, "table")?;
                serialize_table(lua, &bridges, table, table_to_json_string)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "table_to_yaml",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let table = common::expect_table_arg(&args, 0, "table")?;
                serialize_table(lua, &bridges, table, table_to_yaml_string)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "table_to_toml",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let table = common::expect_table_arg(&args, 0, "table")?;
                serialize_table(lua, &bridges, table, table_to_toml_string)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "table_to_csv",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let table = common::expect_table_arg(&args, 0, "table")?;
                serialize_table(lua, &bridges, table, table_to_csv_string)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "table_to_xml",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let table = common::expect_table_arg(&args, 0, "table")?;
                serialize_table(lua, &bridges, table, table_to_xml_string)
            })?,
        )?;
    }

    globals.set(
        "deep_copy",
        lua.create_function(move |lua, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 1)?;
            let table = common::expect_table_arg(&args, 0, "table")?;
            deep_copy_table(lua, &table).map(Value::Table)
        })?,
    )?;

    Ok(())
}

fn serialize_table(
    lua: &Lua,
    bridges: &RuntimeBridges,
    table: Table,
    serializer: impl FnOnce(&JsonValue) -> Result<String, String>,
) -> mlua::Result<Value> {
    match lua_table_to_json(&table).and_then(|json| serializer(&json)) {
        Ok(serialized) => Ok(Value::String(lua.create_string(&serialized)?)),
        Err(reason) => {
            direct_debug_api::write_debug_error_line(bridges, &reason);
            Ok(Value::Nil)
        }
    }
}

fn table_to_json_string(value: &JsonValue) -> Result<String, String> {
    serde_json::to_string(value).map_err(|err| err.to_string())
}

fn table_to_yaml_string(value: &JsonValue) -> Result<String, String> {
    serde_yaml::to_string(value).map_err(|err| err.to_string())
}

fn table_to_toml_string(value: &JsonValue) -> Result<String, String> {
    toml::to_string_pretty(value).map_err(|err| err.to_string())
}

fn table_to_csv_string(value: &JsonValue) -> Result<String, String> {
    let rows = value
        .as_array()
        .ok_or_else(|| "CSV export expects a two-dimensional array table".to_string())?;
    let mut writer = WriterBuilder::new().from_writer(Vec::new());
    for row in rows {
        let columns = row
            .as_array()
            .ok_or_else(|| "CSV export expects each row to be an array".to_string())?;
        let record = columns.iter().map(json_scalar_to_string).collect::<Vec<_>>();
        writer.write_record(record).map_err(|err| err.to_string())?;
    }
    let bytes = writer.into_inner().map_err(|err| err.to_string())?;
    String::from_utf8(bytes).map_err(|err| err.to_string())
}

fn table_to_xml_string(value: &JsonValue) -> Result<String, String> {
    let mut out = String::new();
    value_to_xml("root", value, &mut out);
    Ok(out)
}

fn value_to_xml(tag: &str, value: &JsonValue, out: &mut String) {
    match value {
        JsonValue::Null => {
            out.push('<');
            out.push_str(tag);
            out.push_str("/>");
        }
        JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => {
            out.push('<');
            out.push_str(tag);
            out.push('>');
            out.push_str(&escape_xml(&json_scalar_to_string(value)));
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
        }
        JsonValue::Array(items) => {
            out.push('<');
            out.push_str(tag);
            out.push('>');
            for item in items {
                value_to_xml("item", item, out);
            }
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
        }
        JsonValue::Object(map) => {
            out.push('<');
            out.push_str(tag);
            out.push('>');
            for (key, item) in map {
                value_to_xml(&sanitize_xml_tag(key), item, out);
            }
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
        }
    }
}

fn sanitize_xml_tag(tag: &str) -> String {
    if tag.is_empty() {
        return "item".to_string();
    }
    let mut out = String::new();
    for (idx, ch) in tag.chars().enumerate() {
        let valid = ch.is_ascii_alphanumeric() || ch == '_' || ch == '-';
        if idx == 0 {
            if ch.is_ascii_alphabetic() || ch == '_' {
                out.push(ch);
            } else if valid {
                out.push('_');
                out.push(ch);
            } else {
                out.push('_');
            }
        } else if valid {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() { "item".to_string() } else { out }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn json_scalar_to_string(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => String::new(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        JsonValue::String(value) => value.clone(),
        JsonValue::Array(_) | JsonValue::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn deep_copy_table(lua: &Lua, table: &Table) -> mlua::Result<Table> {
    let new_table = lua.create_table()?;
    for pair in table.clone().pairs::<Value, Value>() {
        let (key, value) = pair?;
        let copied_key = deep_copy_value(lua, key)?;
        let copied_value = deep_copy_value(lua, value)?;
        new_table.set(copied_key, copied_value)?;
    }
    Ok(new_table)
}

fn deep_copy_value(lua: &Lua, value: Value) -> mlua::Result<Value> {
    match value {
        Value::Table(table) => Ok(Value::Table(deep_copy_table(lua, &table)?)),
        other => Ok(other),
    }
}

fn lua_table_to_json(table: &Table) -> Result<JsonValue, String> {
    let mut array_entries = Vec::new();
    let mut object_entries = Map::new();
    let mut array_only = true;

    for pair in table.clone().pairs::<Value, Value>() {
        let (key, value) = pair.map_err(|err| err.to_string())?;
        let json_value = lua_value_to_json(&value)?;
        match key {
            Value::Integer(index) if index >= 1 => {
                array_entries.push((index as usize, json_value));
            }
            Value::String(key) => {
                array_only = false;
                object_entries.insert(
                    key.to_str().map_err(|err| err.to_string())?.to_string(),
                    json_value,
                );
            }
            Value::Nil => {}
            _ => return Err("unsupported lua table key for conversion".to_string()),
        }
    }

    if array_only {
        array_entries.sort_by_key(|(index, _)| *index);
        let contiguous = array_entries
            .iter()
            .enumerate()
            .all(|(expected, (actual, _))| *actual == expected + 1);
        if contiguous {
            return Ok(JsonValue::Array(
                array_entries.into_iter().map(|(_, value)| value).collect(),
            ));
        }
    }

    for (index, value) in array_entries {
        object_entries.insert(index.to_string(), value);
    }
    Ok(JsonValue::Object(object_entries))
}

fn lua_value_to_json(value: &Value) -> Result<JsonValue, String> {
    match value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(value) => Ok(JsonValue::Bool(*value)),
        Value::Integer(value) => Ok(JsonValue::Number(Number::from(*value))),
        Value::Number(value) => Number::from_f64(*value)
            .map(JsonValue::Number)
            .ok_or_else(|| "cannot convert non-finite lua number".to_string()),
        Value::String(value) => Ok(JsonValue::String(
            value.to_str().map_err(|err| err.to_string())?.to_string(),
        )),
        Value::Table(table) => lua_table_to_json(table),
        Value::Function(_) => Err("cannot convert function to string format".to_string()),
        Value::Thread(_) => Err("cannot convert thread to string format".to_string()),
        Value::UserData(_) => Err("cannot convert userdata to string format".to_string()),
        Value::LightUserData(_) => Err("cannot convert lightuserdata to string format".to_string()),
        Value::Error(err) => Err(err.to_string()),
        Value::Other(_) => Err("cannot convert other lua value to string format".to_string()),
    }
}
