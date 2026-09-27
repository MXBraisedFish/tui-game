use std::collections::HashSet;

use mlua::{MultiValue, Table, Value};

pub const MAX_API_STRING_BYTES: usize = 1024 * 1024;
pub const MAX_API_TABLE_ENTRIES: usize = 16_384;

pub fn type_name(value: &Value) -> &'static str {
  match value {
    Value::Nil => "nil",
    Value::Boolean(_) => "boolean",
    Value::LightUserData(_) => "lightuserdata",
    Value::Integer(_) => "integer",
    Value::Number(_) => "float",
    Value::String(_) => "string",
    Value::Table(_) => "table",
    Value::Function(_) => "function",
    Value::Thread(_) => "thread",
    Value::UserData(_) => "userdata",
    Value::Error(_) => "error",
    Value::Other(_) => "other",
  }
}

pub fn invalid(method: &str, name: &str, expected: &str, value: &Value) -> mlua::Error {
  mlua::Error::RuntimeError(format!(
    "{method}: invalid parameter '{name}': expected {expected}, got {}",
    type_name(value)
  ))
}

pub fn message(method: &str, text: impl Into<String>) -> mlua::Error {
  mlua::Error::RuntimeError(format!("{method}: {}", text.into()))
}

pub fn no_args(method: &str, args: MultiValue) -> mlua::Result<()> {
  if args.is_empty() {
    return Ok(());
  }
  if args.len() == 1
    && let Some(Value::Table(table)) = args.front()
    && table.is_empty()
  {
    return Ok(());
  }
  Err(message(method, "expected no parameters"))
}

pub fn one(method: &str, parameter: &str, args: MultiValue) -> mlua::Result<Value> {
  if args.len() != 1 {
    return Err(message(
      method,
      format!("expected one parameter '{parameter}'"),
    ));
  }
  let value = args.into_iter().next().unwrap_or(Value::Nil);
  if let Value::Table(table) = &value
    && table.contains_key(parameter)?
  {
    validate_named_table(method, table, &[parameter])?;
    return table.get(parameter);
  }
  validate_value_limits(method, &value)?;
  Ok(value)
}

pub fn named(method: &str, args: MultiValue, allowed: &[&str]) -> mlua::Result<Table> {
  if args.len() != 1 {
    return Err(message(method, "expected a single named parameter table"));
  }
  let value = args.into_iter().next().unwrap_or(Value::Nil);
  let Value::Table(table) = value else {
    return Err(invalid(method, "parameters", "table", &value));
  };
  validate_named_table(method, &table, allowed)?;
  Ok(table)
}

fn validate_value_limits(method: &str, value: &Value) -> mlua::Result<()> {
  let mut count = 0_usize;
  let mut seen = HashSet::new();
  visit_value_limits(method, value, 0, &mut count, &mut seen)
}

fn validate_named_table(method: &str, table: &Table, allowed: &[&str]) -> mlua::Result<()> {
  let mut count = 0_usize;
  let mut seen = HashSet::new();
  seen.insert(table.to_pointer() as usize);
  for pair in table.clone().pairs::<Value, Value>() {
    let (key, value) = pair?;
    count = count.saturating_add(1);
    if count > MAX_API_TABLE_ENTRIES {
      return Err(message(method, "parameter table exceeds 16384 entries"));
    }
    let Value::String(key) = key else {
      return Err(message(method, "named parameter keys must be strings"));
    };
    let key = key.to_str()?;
    if !allowed.iter().any(|allowed| *allowed == key.as_ref()) {
      return Err(message(method, format!("unknown parameter '{}'", key)));
    }
    visit_value_limits(method, &value, 1, &mut count, &mut seen)?;
  }
  Ok(())
}

fn visit_value_limits(
  method: &str,
  value: &Value,
  depth: usize,
  count: &mut usize,
  seen: &mut HashSet<usize>,
) -> mlua::Result<()> {
  let Value::Table(table) = value else {
    return Ok(());
  };
  if depth >= 32 {
    return Err(message(method, "parameter table exceeds 32 levels"));
  }
  let pointer = table.to_pointer() as usize;
  if !seen.insert(pointer) {
    return Err(message(method, "parameter table contains a cycle"));
  }
  for pair in table.clone().pairs::<Value, Value>() {
    let (key, value) = pair?;
    *count = count.saturating_add(1);
    if *count > MAX_API_TABLE_ENTRIES {
      return Err(message(method, "parameter table exceeds 16384 entries"));
    }
    visit_value_limits(method, &key, depth + 1, count, seen)?;
    visit_value_limits(method, &value, depth + 1, count, seen)?;
  }
  seen.remove(&pointer);
  Ok(())
}

pub fn required(table: &Table, method: &str, name: &str) -> mlua::Result<Value> {
  let value = table.get::<Value>(name)?;
  if matches!(value, Value::Nil) {
    Err(invalid(method, name, "non-nil value", &value))
  } else {
    Ok(value)
  }
}

pub fn string(value: Value, method: &str, name: &str) -> mlua::Result<String> {
  let Value::String(value) = value else {
    return Err(invalid(method, name, "UTF-8 string", &value));
  };
  let value = value.to_str().map_err(|_| {
    invalid(
      method,
      name,
      "valid UTF-8 string",
      &Value::String(value.clone()),
    )
  })?;
  if value.len() > MAX_API_STRING_BYTES {
    return Err(message(method, format!("parameter '{name}' exceeds 1 MiB")));
  }
  Ok(value.to_string())
}

/// Converts a free-form Lua value into user-facing text using the same stable
/// representation exposed by `base.tostring`.
///
/// This is intentionally separate from [`string`]: identifiers, paths,
/// constants, encodings and protocol strings must continue to require an
/// actual UTF-8 Lua string.
pub fn dynamic_text(value: Value, method: &str, name: &str) -> mlua::Result<String> {
  let text = match value {
    Value::Nil => "nil".to_string(),
    Value::Boolean(value) => value.to_string(),
    Value::Integer(value) => value.to_string(),
    Value::Number(value) => value.to_string(),
    Value::String(value) => value
      .to_str()
      .map_err(|_| {
        invalid(
          method,
          name,
          "value convertible to a valid UTF-8 string",
          &Value::String(value.clone()),
        )
      })?
      .to_string(),
    Value::Table(value) => format!("table: {:p}", value.to_pointer()),
    Value::Function(value) => format!("function: {:p}", value.to_pointer()),
    Value::Thread(value) => format!("thread: {:p}", value.to_pointer()),
    Value::UserData(value) => format!("userdata: {:p}", value.to_pointer()),
    value => type_name(&value).to_string(),
  };
  if text.len() > MAX_API_STRING_BYTES {
    return Err(message(method, format!("parameter '{name}' exceeds 1 MiB")));
  }
  Ok(text)
}

pub fn integer(value: Value, method: &str, name: &str) -> mlua::Result<i64> {
  match value {
    Value::Integer(value) => Ok(value),
    Value::Number(value) if value.is_finite() && value.fract() == 0.0 => {
      if value < i64::MIN as f64 || value >= 9_223_372_036_854_775_808.0 {
        Err(invalid(method, name, "integer", &Value::Number(value)))
      } else {
        Ok(value as i64)
      }
    }
    value => Err(invalid(method, name, "integer", &value)),
  }
}

pub fn number(value: Value, method: &str, name: &str) -> mlua::Result<f64> {
  match value {
    Value::Integer(value) => Ok(value as f64),
    Value::Number(value) => Ok(value),
    value => Err(invalid(method, name, "float or integer", &value)),
  }
}

pub fn boolean(value: Value, method: &str, name: &str) -> mlua::Result<bool> {
  let Value::Boolean(value) = value else {
    return Err(invalid(method, name, "boolean", &value));
  };
  Ok(value)
}

pub fn values(table: &Table, method: &str) -> mlua::Result<Vec<Value>> {
  let value = required(table, method, "values")?;
  array_values(value, method, "values")
}

pub fn array_values(value: Value, method: &str, name: &str) -> mlua::Result<Vec<Value>> {
  let Value::Table(values) = value else {
    return Err(invalid(method, name, "array table", &value));
  };
  let declared_length = values.raw_get::<Value>("n")?;
  let length = if matches!(declared_length, Value::Nil) {
    values.raw_len()
  } else {
    usize::try_from(integer(declared_length, method, &format!("{name}.n"))?)
      .map_err(|_| message(method, format!("{name}.n must be a non-negative integer")))?
  };
  if length > MAX_API_TABLE_ENTRIES {
    return Err(message(method, format!("{name} exceeds 16384 entries")));
  }
  (1..=length).map(|index| values.raw_get(index)).collect()
}

pub fn optional_integer(
  table: &Table,
  method: &str,
  name: &str,
  default: Option<i64>,
) -> mlua::Result<Option<i64>> {
  let value = table.get::<Value>(name)?;
  if matches!(value, Value::Nil) {
    Ok(default)
  } else {
    integer(value, method, name).map(Some)
  }
}

pub fn optional_string(
  table: &Table,
  method: &str,
  name: &str,
  default: Option<&str>,
) -> mlua::Result<Option<String>> {
  let value = table.get::<Value>(name)?;
  if matches!(value, Value::Nil) {
    Ok(default.map(ToOwned::to_owned))
  } else {
    string(value, method, name).map(Some)
  }
}

pub fn optional_dynamic_text(
  table: &Table,
  method: &str,
  name: &str,
  default: Option<&str>,
) -> mlua::Result<Option<String>> {
  let value = table.get::<Value>(name)?;
  if matches!(value, Value::Nil) {
    Ok(default.map(ToOwned::to_owned))
  } else {
    dynamic_text(value, method, name).map(Some)
  }
}

pub fn optional_bool(table: &Table, method: &str, name: &str, default: bool) -> mlua::Result<bool> {
  let value = table.get::<Value>(name)?;
  if matches!(value, Value::Nil) {
    Ok(default)
  } else {
    boolean(value, method, name)
  }
}
