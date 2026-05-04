//! Lua 表到 JSON 中间值转换

use mlua::{Table, Value};
use serde_json::{Map, Number, Value as JsonValue};

/// 将 Lua 表转换为 JSON 中间值。
pub fn lua_table_to_json(table: &Table) -> mlua::Result<JsonValue> {
    let mut array_entries = Vec::new();
    let mut object_entries = Map::new();
    let mut array_only = true;

    for pair in table.clone().pairs::<Value, Value>() {
        let (key, value) = pair.map_err(mlua::Error::external)?;
        let json_value = lua_value_to_json(&value)?;
        match key {
            Value::Integer(index) if index >= 1 => {
                array_entries.push((index as usize, json_value));
            }
            Value::String(key) => {
                array_only = false;
                object_entries.insert(key.to_str()?.to_string(), json_value);
            }
            Value::Nil => {}
            _ => {
                return Err(mlua::Error::external(
                    "table key must be a positive integer or string",
                ));
            }
        }
    }

    if array_only {
        array_entries.sort_by_key(|(index, _)| *index);
        let contiguous = array_entries
            .iter()
            .enumerate()
            .all(|(expected_index, (actual_index, _))| *actual_index == expected_index + 1);
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

fn lua_value_to_json(value: &Value) -> mlua::Result<JsonValue> {
    match value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(value) => Ok(JsonValue::Bool(*value)),
        Value::Integer(value) => Ok(JsonValue::Number(Number::from(*value))),
        Value::Number(value) => Number::from_f64(*value)
            .map(JsonValue::Number)
            .ok_or_else(|| mlua::Error::external("number is not finite")),
        Value::String(value) => Ok(JsonValue::String(value.to_str()?.to_string())),
        Value::Table(table) => lua_table_to_json(table),
        Value::Function(_)
        | Value::Thread(_)
        | Value::UserData(_)
        | Value::LightUserData(_)
        | Value::Other(_)
        | Value::Error(_) => Err(mlua::Error::external(
            "table contains unsupported lua value type",
        )),
    }
}
