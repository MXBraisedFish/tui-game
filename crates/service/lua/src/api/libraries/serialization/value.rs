use std::collections::{BTreeMap, HashSet};

use mlua::{Lua, Table, Value};

use super::super::{args, readonly};

const MAX_DEPTH: usize = 32;
const MAX_NODES: usize = 16_384;

pub(super) fn lua_to_json(value: Value, method: &str) -> mlua::Result<serde_json::Value> {
  let mut seen = HashSet::new();
  let mut nodes = 0;
  encode(value, method, 0, &mut nodes, &mut seen)
}

fn encode(
  value: Value,
  method: &str,
  depth: usize,
  nodes: &mut usize,
  seen: &mut HashSet<usize>,
) -> mlua::Result<serde_json::Value> {
  *nodes = nodes.saturating_add(1);
  if *nodes > MAX_NODES {
    return Err(args::message(method, "value exceeds 16384 entries"));
  }
  match value {
    Value::Nil => Ok(serde_json::Value::Null),
    Value::Boolean(value) => Ok(value.into()),
    Value::Integer(value) => Ok(value.into()),
    Value::Number(value) if value.is_finite() => serde_json::Number::from_f64(value)
      .map(serde_json::Value::Number)
      .ok_or_else(|| args::message(method, "number is not finite")),
    Value::Number(_) => Err(args::message(
      method,
      "non-finite numbers cannot be serialized",
    )),
    Value::String(value) => Ok(serde_json::Value::String(
      value
        .to_str()
        .map_err(|_| args::message(method, "text formats require valid UTF-8 strings"))?
        .to_string(),
    )),
    Value::Table(table) => encode_table(table, method, depth, nodes, seen),
    value => Err(args::message(
      method,
      format!("{} values cannot be serialized", args::type_name(&value)),
    )),
  }
}

fn encode_table(
  table: Table,
  method: &str,
  depth: usize,
  nodes: &mut usize,
  seen: &mut HashSet<usize>,
) -> mlua::Result<serde_json::Value> {
  if depth >= MAX_DEPTH {
    return Err(args::message(method, "value exceeds 32 levels"));
  }
  let table = readonly::backing(&table)?;
  let pointer = table.to_pointer() as usize;
  if !seen.insert(pointer) {
    return Err(args::message(method, "cyclic tables cannot be serialized"));
  }
  let result = (|| {
    let mut integer_values = BTreeMap::new();
    let mut object = serde_json::Map::new();
    let mut has_integer = false;
    let mut has_string = false;
    for pair in table.clone().pairs::<Value, Value>() {
      let (key, value) = pair?;
      match key {
        Value::Integer(index) if index > 0 => {
          has_integer = true;
          integer_values.insert(index as usize, value);
        }
        Value::String(key) => {
          has_string = true;
          let key = key
            .to_str()
            .map_err(|_| args::message(method, "object keys must be valid UTF-8"))?
            .to_string();
          object.insert(key, encode(value, method, depth + 1, nodes, seen)?);
        }
        key => {
          return Err(args::message(
            method,
            format!(
              "unsupported table key type '{}'; expected array indexes or strings",
              args::type_name(&key)
            ),
          ));
        }
      }
    }
    if has_integer && has_string {
      return Err(args::message(
        method,
        "mixed array and object tables cannot be serialized",
      ));
    }
    if has_integer {
      let length = *integer_values.keys().next_back().unwrap_or(&0);
      if integer_values.len() != length {
        return Err(args::message(method, "sparse arrays cannot be serialized"));
      }
      let mut array = Vec::with_capacity(length);
      for (_, value) in integer_values {
        array.push(encode(value, method, depth + 1, nodes, seen)?);
      }
      Ok(serde_json::Value::Array(array))
    } else {
      Ok(serde_json::Value::Object(object))
    }
  })();
  seen.remove(&pointer);
  result
}

pub(super) fn json_to_lua(
  lua: &Lua,
  value: &serde_json::Value,
  method: &str,
) -> mlua::Result<Value> {
  fn decode(
    lua: &Lua,
    value: &serde_json::Value,
    method: &str,
    depth: usize,
    nodes: &mut usize,
  ) -> mlua::Result<Value> {
    *nodes = nodes.saturating_add(1);
    if *nodes > MAX_NODES || depth > MAX_DEPTH {
      return Err(args::message(method, "decoded value exceeds safety limits"));
    }
    match value {
      serde_json::Value::Null => Ok(Value::Nil),
      serde_json::Value::Bool(value) => Ok(Value::Boolean(*value)),
      serde_json::Value::Number(value) => {
        if let Some(value) = value.as_i64() {
          Ok(Value::Integer(value))
        } else {
          Ok(Value::Number(
            value
              .as_f64()
              .ok_or_else(|| args::message(method, "invalid number"))?,
          ))
        }
      }
      serde_json::Value::String(value) => Ok(Value::String(lua.create_string(value)?)),
      serde_json::Value::Array(values) => {
        let table = lua.create_table_with_capacity(values.len(), 0)?;
        for (index, value) in values.iter().enumerate() {
          table.raw_set(index + 1, decode(lua, value, method, depth + 1, nodes)?)?;
        }
        Ok(Value::Table(table))
      }
      serde_json::Value::Object(values) => {
        let table = lua.create_table_with_capacity(0, values.len())?;
        for (key, value) in values {
          table.raw_set(key.as_str(), decode(lua, value, method, depth + 1, nodes)?)?;
        }
        Ok(Value::Table(table))
      }
    }
  }
  decode(lua, value, method, 0, &mut 0)
}

pub(super) fn text_argument(values: mlua::MultiValue, method: &str) -> mlua::Result<String> {
  let value = args::one(method, "s", values)?;
  args::string(value, method, "s")
}

pub(super) fn bounded_text(method: &str, value: String) -> mlua::Result<String> {
  if value.len() > args::MAX_API_STRING_BYTES {
    Err(args::message(method, "serialized output exceeds 1 MiB"))
  } else {
    Ok(value)
  }
}
