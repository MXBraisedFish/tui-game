use mlua::{Lua, LuaString, MultiValue, Table, Value};

pub const MAX_API_STRING_BYTES: usize = 1024 * 1024;
pub const MAX_API_TABLE_ENTRIES: usize = 16_384;

/// The parsed positional values and a sanitized optional-parameter table.
///
/// Positional values stay separate from options so a data table in a required
/// slot is never mistaken for an options table, and explicit nil arguments
/// remain distinguishable from omitted arguments through [`Self::was_supplied`].
/// Positional payloads and nested option values are left to their API's own
/// validation so generic parsing does not reinterpret user data structures.
#[derive(Debug)]
pub struct PositionalArgs {
  values: Vec<Value>,
  options: Table,
}

impl PositionalArgs {
  pub fn get(&self, index: usize) -> Value {
    self.values.get(index).cloned().unwrap_or(Value::Nil)
  }

  pub fn required(&self, index: usize, method: &str, name: &str) -> mlua::Result<Value> {
    if !self.was_supplied(index) {
      return Err(message(
        method,
        format!(
          "missing required parameter '{name}' at position {}",
          index + 1
        ),
      ));
    }
    let value = self.get(index);
    if matches!(value, Value::Nil) {
      Err(invalid(method, name, "non-nil value", &value))
    } else {
      Ok(value)
    }
  }

  pub fn was_supplied(&self, index: usize) -> bool {
    index < self.values.len()
  }

  pub fn options(&self) -> &Table {
    &self.options
  }
}

/// Parses a fixed positional prefix followed by at most one strict options
/// table. If `option_fields` is empty, no extra arguments are accepted.
///
/// A final nil is accepted as an omitted options table only when the method
/// declares option fields. Nil values inside an options table are naturally
/// absent in Lua and therefore use the method's normal defaults.
pub fn positional(
  lua: &Lua,
  method: &str,
  arguments: MultiValue,
  required_names: &[&str],
  option_fields: &[&str],
) -> mlua::Result<PositionalArgs> {
  let values = arguments.into_iter().collect::<Vec<_>>();
  let required_count = required_names.len();
  if values.len() < required_count {
    let index = values.len();
    return Err(message(
      method,
      format!(
        "missing required parameter '{}' at position {}",
        required_names[index],
        index + 1
      ),
    ));
  }

  let mut options = lua.create_table()?;
  if values.len() > required_count {
    if option_fields.is_empty() {
      return Err(message(
        method,
        format!(
          "expected {required_count} argument(s), got {}",
          values.len()
        ),
      ));
    }
    if values.len() != required_count + 1 {
      return Err(message(
        method,
        format!(
          "expected {required_count} positional argument(s) and at most one trailing options table, got {} arguments",
          values.len()
        ),
      ));
    }
    match &values[required_count] {
      Value::Nil => {}
      Value::Table(input) => {
        if input.metatable().is_some() {
          return Err(message(method, "options table must not have a metatable"));
        }
        copy_options(method, input, &mut options, option_fields)?;
      }
      value => {
        return Err(invalid(method, "options", "table or nil", value));
      }
    }
  }

  Ok(PositionalArgs {
    values: values[..required_count].to_vec(),
    options,
  })
}

/// Validates a required prefix while preserving every following value as a
/// method argument. Use this for project variadic methods that do not declare
/// an options table.
pub fn variadic(
  method: &str,
  arguments: MultiValue,
  required_names: &[&str],
) -> mlua::Result<Vec<Value>> {
  let values = arguments.into_iter().collect::<Vec<_>>();
  if values.len() < required_names.len() {
    let index = values.len();
    return Err(message(
      method,
      format!(
        "missing required parameter '{}' at position {}",
        required_names[index],
        index + 1
      ),
    ));
  }
  Ok(values)
}

fn copy_options(
  method: &str,
  input: &Table,
  output: &mut Table,
  allowed: &[&str],
) -> mlua::Result<()> {
  let mut count = 0_usize;
  for pair in input.clone().pairs::<Value, Value>() {
    let (key, value) = pair?;
    count = count.saturating_add(1);
    if count > MAX_API_TABLE_ENTRIES {
      return Err(message(method, "options table exceeds 16384 entries"));
    }
    let Value::String(key) = key else {
      return Err(message(method, "option names must be strings"));
    };
    let key = key.to_str()?;
    if !allowed.iter().any(|field| *field == key.as_ref()) {
      return Err(message(method, format!("unknown option '{key}'")));
    }
    output.raw_set(key.as_ref(), value)?;
  }
  Ok(())
}

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

pub fn no_args(method: &str, arguments: MultiValue) -> mlua::Result<()> {
  if arguments.is_empty() {
    Ok(())
  } else {
    Err(message(
      method,
      format!("expected no arguments, got {}", arguments.len()),
    ))
  }
}

pub fn one(method: &str, parameter: &str, args: MultiValue) -> mlua::Result<Value> {
  if args.len() != 1 {
    return Err(message(
      method,
      format!("expected one parameter '{parameter}'"),
    ));
  }
  let value = args.into_iter().next().unwrap_or(Value::Nil);
  Ok(value)
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
  Ok(lua_string(value, method, name)?.to_str()?.to_string())
}

pub fn lua_string(value: Value, method: &str, name: &str) -> mlua::Result<LuaString> {
  let Value::String(value) = value else {
    return Err(invalid(method, name, "UTF-8 string", &value));
  };
  let length = value
    .to_str()
    .map_err(|_| {
      invalid(
        method,
        name,
        "valid UTF-8 string",
        &Value::String(value.clone()),
      )
    })?
    .len();
  if length > MAX_API_STRING_BYTES {
    return Err(message(method, format!("parameter '{name}' exceeds 1 MiB")));
  }
  Ok(value)
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

#[cfg(test)]
mod tests {
  use super::*;

  fn one_integer(value: i64) -> MultiValue {
    MultiValue::from_vec(vec![Value::Integer(value)])
  }

  #[test]
  fn positional_keeps_required_tables_as_data_and_preserves_explicit_nil() {
    let lua = Lua::new();
    let data = lua.create_table().unwrap();
    data.raw_set("text", "payload").unwrap();
    let options = lua.create_table().unwrap();
    options.raw_set("enabled", false).unwrap();
    options.raw_set("count", 0).unwrap();
    options.raw_set("label", "").unwrap();

    let parsed = positional(
      &lua,
      "test.method",
      MultiValue::from_vec(vec![Value::Table(data.clone()), Value::Table(options)]),
      &["data"],
      &["enabled", "count", "label"],
    )
    .unwrap();
    let Value::Table(actual) = parsed.get(0) else {
      panic!("required table argument was not preserved");
    };
    assert_eq!(actual.to_pointer(), data.to_pointer());
    assert!(parsed.was_supplied(0));
    assert!(!parsed.was_supplied(1));
    assert!(!parsed.options().get::<bool>("enabled").unwrap());
    assert_eq!(parsed.options().get::<i64>("count").unwrap(), 0);
    assert_eq!(parsed.options().get::<String>("label").unwrap(), "");

    let explicit_nil = positional(
      &lua,
      "test.method",
      MultiValue::from_vec(vec![Value::Nil]),
      &["value"],
      &[],
    )
    .unwrap();
    assert!(explicit_nil.was_supplied(0));
    assert!(matches!(explicit_nil.get(0), Value::Nil));
  }

  #[test]
  fn one_treats_a_table_as_positional_data_without_named_unwrapping() {
    let lua = Lua::new();
    let table = lua.create_table().unwrap();
    table.raw_set("value", 42).unwrap();
    let value = one(
      "test.method",
      "value",
      MultiValue::from_vec(vec![Value::Table(table.clone())]),
    )
    .unwrap();
    let Value::Table(actual) = value else {
      panic!("table data was not preserved");
    };
    assert_eq!(actual.to_pointer(), table.to_pointer());
  }

  #[test]
  fn positional_parsing_does_not_reject_cyclic_payload_or_nested_option_data() {
    let lua = Lua::new();
    let payload: Table = lua
      .load("local t = {}; t.self = t; return t")
      .eval()
      .unwrap();
    let nested_options: Table = lua
      .load("local t = {}; t.self = t; return t")
      .eval()
      .unwrap();
    let options = lua.create_table().unwrap();
    options.raw_set("metadata", nested_options.clone()).unwrap();
    let parsed = positional(
      &lua,
      "test.method",
      MultiValue::from_vec(vec![Value::Table(payload.clone()), Value::Table(options)]),
      &["payload"],
      &["metadata"],
    )
    .unwrap();
    assert!(
      matches!(parsed.get(0), Value::Table(value) if value.to_pointer() == payload.to_pointer())
    );
    assert!(matches!(
      parsed.options().get::<Value>("metadata").unwrap(),
      Value::Table(value) if value.to_pointer() == nested_options.to_pointer()
    ));
  }

  #[test]
  fn variadic_keeps_table_and_nil_values_after_the_required_prefix() {
    let lua = Lua::new();
    let table = lua.create_table().unwrap();
    table.raw_set("payload", true).unwrap();
    let values = variadic(
      "test.format",
      MultiValue::from_vec(vec![
        Value::String(lua.create_string("%s").unwrap()),
        Value::Table(table.clone()),
        Value::Nil,
      ]),
      &["format"],
    )
    .unwrap();
    assert_eq!(values.len(), 3);
    assert!(matches!(&values[1], Value::Table(value) if value.to_pointer() == table.to_pointer()));
    assert!(matches!(values[2], Value::Nil));
  }

  #[test]
  fn positional_accepts_omitted_empty_and_nil_options_equivalently() {
    let lua = Lua::new();
    let omitted = positional(
      &lua,
      "test.method",
      one_integer(7),
      &["value"],
      &["enabled"],
    )
    .unwrap();
    let empty = lua.create_table().unwrap();
    let explicit = positional(
      &lua,
      "test.method",
      MultiValue::from_vec(vec![Value::Integer(7), Value::Table(empty)]),
      &["value"],
      &["enabled"],
    )
    .unwrap();
    let nil = positional(
      &lua,
      "test.method",
      MultiValue::from_vec(vec![Value::Integer(7), Value::Nil]),
      &["value"],
      &["enabled"],
    )
    .unwrap();
    assert!(matches!(
      omitted.options().get::<Value>("enabled").unwrap(),
      Value::Nil
    ));
    assert!(matches!(
      explicit.options().get::<Value>("enabled").unwrap(),
      Value::Nil
    ));
    assert!(matches!(
      nil.options().get::<Value>("enabled").unwrap(),
      Value::Nil
    ));
  }

  #[test]
  fn positional_rejects_missing_surplus_and_non_table_options() {
    let lua = Lua::new();
    let missing = positional(&lua, "test.method", MultiValue::new(), &["value"], &[]).unwrap_err();
    assert!(missing.to_string().contains("value"));
    assert!(missing.to_string().contains("position 1"));

    let surplus = positional(
      &lua,
      "test.method",
      MultiValue::from_vec(vec![Value::Integer(1), Value::Nil]),
      &["value"],
      &[],
    )
    .unwrap_err();
    assert!(surplus.to_string().contains("got 2"));

    let wrong_options = positional(
      &lua,
      "test.method",
      MultiValue::from_vec(vec![Value::Integer(1), Value::Boolean(true)]),
      &["value"],
      &["enabled"],
    )
    .unwrap_err();
    assert!(wrong_options.to_string().contains("table or nil"));

    let too_many = positional(
      &lua,
      "test.method",
      MultiValue::from_vec(vec![
        Value::Integer(1),
        Value::Table(lua.create_table().unwrap()),
        Value::Nil,
      ]),
      &["value"],
      &["enabled"],
    )
    .unwrap_err();
    assert!(
      too_many
        .to_string()
        .contains("at most one trailing options table")
    );
  }

  #[test]
  fn positional_options_reject_unknown_numeric_and_metatable_fields() {
    let lua = Lua::new();
    let unknown = lua.create_table().unwrap();
    unknown.raw_set("enabeld", true).unwrap();
    let error = positional(
      &lua,
      "test.method",
      MultiValue::from_vec(vec![Value::Table(unknown)]),
      &[],
      &["enabled"],
    )
    .unwrap_err();
    assert!(error.to_string().contains("unknown option 'enabeld'"));

    let numeric = lua.create_table().unwrap();
    numeric.raw_set(1, true).unwrap();
    let error = positional(
      &lua,
      "test.method",
      MultiValue::from_vec(vec![Value::Table(numeric)]),
      &[],
      &["enabled"],
    )
    .unwrap_err();
    assert!(error.to_string().contains("option names must be strings"));

    let metatable: Table = lua
      .load("return setmetatable({}, { __index = { enabled = true } })")
      .eval()
      .unwrap();
    let error = positional(
      &lua,
      "test.method",
      MultiValue::from_vec(vec![Value::Table(metatable)]),
      &[],
      &["enabled"],
    )
    .unwrap_err();
    assert!(error.to_string().contains("must not have a metatable"));
  }
}
