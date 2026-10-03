//! Lua base library bindings with validated arguments and session-owned host access.

use super::*;

/// Build and register the Lua base API in the supplied VM and host context.
///
/// # Errors
///
/// Propagate Lua allocation, table construction, or function registration errors while installing
/// this library.
pub(super) fn base(lua: &Lua) -> mlua::Result<Table> {
  let ipairs = lua.create_function(|lua, args: MultiValue| {
    let value = positional_argument(&args, 0, "base.ipairs", "table")?;
    let Value::Table(table) = value else {
      return Err(args::invalid("base.ipairs", "table", "table", &value));
    };
    let iterator = lua.create_function(|_, args: MultiValue| {
      let state = positional_argument(&args, 0, "base.ipairs iterator", "state")?;
      let index = positional_argument(&args, 1, "base.ipairs iterator", "index")?;
      let Value::Table(state) = state else {
        return Err(args::invalid(
          "base.ipairs iterator",
          "state",
          "table",
          &state,
        ));
      };
      let index = args::integer(index, "base.ipairs iterator", "index")?;
      let next = index
        .checked_add(1)
        .ok_or_else(|| args::message("base.ipairs iterator", "index overflow"))?;
      let value = state.get::<Value>(next)?;
      if matches!(value, Value::Nil) {
        Ok(MultiValue::from_vec(vec![Value::Nil]))
      } else {
        Ok(MultiValue::from_vec(vec![Value::Integer(next), value]))
      }
    })?;
    Ok(MultiValue::from_vec(vec![
      Value::Function(iterator),
      Value::Table(table),
      Value::Integer(0),
    ]))
  })?;
  let next = lua.create_function(|_, args: MultiValue| {
    let value = positional_argument(&args, 0, "base.next", "table")?;
    let Value::Table(table) = value else {
      return Err(args::invalid("base.next", "table", "table", &value));
    };
    let index = args.get(1).cloned().unwrap_or(Value::Nil);
    next_pair(&table, index)
  })?;
  let next_for_pairs = next.clone();
  let pairs = lua.create_function(move |_, args: MultiValue| {
    let value = positional_argument(&args, 0, "base.pairs", "table")?;
    let Value::Table(table) = value else {
      return Err(args::invalid("base.pairs", "table", "table", &value));
    };
    if let Some(metatable) = table.metatable() {
      let metamethod = metatable.raw_get::<Value>("__pairs")?;
      if let Value::Function(metamethod) = metamethod {
        let results = metamethod.call::<MultiValue>(table)?;
        let mut values = results.into_iter();
        return Ok(MultiValue::from_vec(vec![
          values.next().unwrap_or(Value::Nil),
          values.next().unwrap_or(Value::Nil),
          values.next().unwrap_or(Value::Nil),
        ]));
      }
      if !matches!(metamethod, Value::Nil) {
        return Err(args::invalid(
          "base.pairs",
          "__pairs",
          "function",
          &metamethod,
        ));
      }
    }
    Ok(MultiValue::from_vec(vec![
      Value::Function(next_for_pairs.clone()),
      Value::Table(table),
      Value::Nil,
    ]))
  })?;
  let select = lua.create_function(|_, args: MultiValue| {
    let index = positional_argument(&args, 0, "base.select", "index")?;
    let values = args.into_iter().skip(1).collect::<Vec<_>>();
    if let Value::String(index) = &index
      && index.to_str()?.as_ref() == "#"
    {
      return Ok(MultiValue::from_vec(vec![Value::Integer(
        values.len() as i64
      )]));
    }
    let index = args::integer(index, "base.select", "index")?;
    let start = if index < 0 {
      values.len() as i64 + index + 1
    } else {
      index
    };
    if start < 1 || start > values.len() as i64 + 1 {
      return Err(args::message("base.select", "index out of range"));
    }
    Ok(MultiValue::from_vec(
      values.into_iter().skip((start - 1) as usize).collect(),
    ))
  })?;
  let rawequal = lua.create_function(|_, args: MultiValue| {
    let left = args.front().cloned().unwrap_or(Value::Nil);
    let right = args.get(1).cloned().unwrap_or(Value::Nil);
    Ok(raw_equal(&left, &right))
  })?;
  let rawget = lua.create_function(|_, args: MultiValue| {
    let value = positional_argument(&args, 0, "base.rawget", "table")?;
    let Value::Table(table) = value else {
      return Err(args::invalid("base.rawget", "table", "table", &value));
    };
    let key = positional_argument(&args, 1, "base.rawget", "key")?;
    table.raw_get::<Value>(key)
  })?;
  let rawset = lua.create_function(|_, args: MultiValue| {
    let value = positional_argument(&args, 0, "base.rawset", "table")?;
    let Value::Table(table) = value else {
      return Err(args::invalid("base.rawset", "table", "table", &value));
    };
    if readonly::is_proxy(&table)? {
      return Err(mlua::Error::RuntimeError(
        "attempt to modify a read-only TUI GAME API table".to_string(),
      ));
    }
    let key = positional_argument(&args, 1, "base.rawset", "key")?;
    let value = args.get(2).cloned().unwrap_or(Value::Nil);
    table.raw_set(key, value)?;
    Ok(table)
  })?;
  let rawlen = lua.create_function(|_, args: MultiValue| {
    let value = positional_argument(&args, 0, "base.rawlen", "value")?;
    match value {
      Value::String(value) => Ok(value.as_bytes().len() as i64),
      Value::Table(table) => Ok(table.raw_len() as i64),
      value => Err(args::invalid(
        "base.rawlen",
        "value",
        "string or table",
        &value,
      )),
    }
  })?;
  let tonumber = lua.create_function(|_, args: MultiValue| {
    let value = positional_argument(&args, 0, "base.tonumber", "value")?;
    let base = args.get(1).cloned().unwrap_or(Value::Nil);
    let base = if matches!(base, Value::Nil) {
      None
    } else {
      Some(args::integer(base, "base.tonumber", "base")?)
    };
    match (value, base) {
      (Value::Integer(value), None) => Ok(Value::Integer(value)),
      (Value::Number(value), None) => Ok(Value::Number(value)),
      (Value::String(value), None) => {
        let text = value.to_str()?;
        Ok(parse_number(text.as_ref()).unwrap_or(Value::Nil))
      }
      (Value::String(value), Some(base @ 2..=36)) => {
        let text = value.to_str()?;
        Ok(
          i64::from_str_radix(text.trim(), base as u32)
            .map(Value::Integer)
            .unwrap_or(Value::Nil),
        )
      }
      (value, Some(_)) if !matches!(value, Value::String(_)) => Err(args::invalid(
        "base.tonumber",
        "value",
        "string when a base is provided",
        &value,
      )),
      (_, Some(_)) => Err(args::message(
        "base.tonumber",
        "base must be in the range 2..36",
      )),
      _ => Ok(Value::Nil),
    }
  })?;
  let tostring = lua.create_function(|lua, args: MultiValue| {
    let value = positional_argument(&args, 0, "base.tostring", "value")?;
    if let Value::Table(table) = &value
      && let Some(metatable) = table.metatable()
    {
      let metamethod = metatable.raw_get::<Value>("__tostring")?;
      if !matches!(metamethod, Value::Nil) {
        let Value::Function(metamethod) = metamethod else {
          return Err(args::invalid(
            "base.tostring",
            "__tostring",
            "function",
            &metamethod,
          ));
        };
        let result = metamethod.call::<Value>(table.clone())?;
        return args::string(result, "base.tostring", "__tostring result")
          .and_then(|text| lua.create_string(text));
      }
      if let Value::String(name) = metatable.raw_get::<Value>("__name")? {
        return lua.create_string(format!(
          "{}: {:p}",
          name.to_string_lossy(),
          table.to_pointer()
        ));
      }
    }
    let text = args::dynamic_text(value, "base.tostring", "value")?;
    lua.create_string(text)
  })?;
  let type_fn = lua.create_function(|lua, args: MultiValue| {
    let value = positional_argument(&args, 0, "base.type", "value")?;
    let type_name = match value {
      Value::Integer(_) | Value::Number(_) => "number",
      _ => args::type_name(&value),
    };
    lua.create_string(type_name)
  })?;
  let setmetatable = lua.create_function(|_, args: MultiValue| {
    let target = positional_argument(&args, 0, "base.setmetatable", "table")?;
    let Value::Table(target) = target else {
      return Err(args::invalid(
        "base.setmetatable",
        "table",
        "table",
        &target,
      ));
    };
    if let Some(current) = target.metatable() {
      let protection = current.raw_get::<Value>("__metatable")?;
      if !matches!(protection, Value::Nil) {
        return Err(args::message(
          "base.setmetatable",
          "cannot change a protected metatable",
        ));
      }
    }
    let metatable = match positional_argument(&args, 1, "base.setmetatable", "metatable")? {
      Value::Table(metatable) => Some(metatable),
      Value::Nil => None,
      value => {
        return Err(args::invalid(
          "base.setmetatable",
          "metatable",
          "table or nil",
          &value,
        ));
      }
    };
    target.set_metatable(metatable)?;
    Ok(target)
  })?;
  let getmetatable = lua.create_function(|_, args: MultiValue| {
    let value = positional_argument(&args, 0, "base.getmetatable", "value")?;
    let Value::Table(target) = value else {
      return Ok(Value::Nil);
    };
    let Some(metatable) = target.metatable() else {
      return Ok(Value::Nil);
    };
    let protection = metatable.raw_get::<Value>("__metatable")?;
    if matches!(protection, Value::Nil) {
      Ok(Value::Table(metatable))
    } else {
      Ok(protection)
    }
  })?;
  readonly::library(
    lua,
    [
      ("ipairs", function_value(ipairs)),
      ("pairs", function_value(pairs)),
      ("next", function_value(next)),
      ("select", function_value(select)),
      ("rawequal", function_value(rawequal)),
      ("rawget", function_value(rawget)),
      ("rawset", function_value(rawset)),
      ("rawlen", function_value(rawlen)),
      ("tonumber", function_value(tonumber)),
      ("tostring", function_value(tostring)),
      ("type", function_value(type_fn)),
      ("setmetatable", function_value(setmetatable)),
      ("getmetatable", function_value(getmetatable)),
    ],
  )
}

fn positional_argument(
  arguments: &MultiValue,
  index: usize,
  method: &str,
  name: &str,
) -> mlua::Result<Value> {
  arguments
    .get(index)
    .cloned()
    .ok_or_else(|| args::invalid(method, name, "value", &Value::Nil))
}

fn next_pair(table: &Table, index: Value) -> mlua::Result<MultiValue> {
  let source = readonly::backing(table)?;
  let mut found = matches!(index, Value::Nil);
  for pair in source.pairs::<Value, Value>() {
    let (key, value) = pair?;
    if found {
      return Ok(MultiValue::from_vec(vec![key, value]));
    }
    if raw_equal(&key, &index) {
      found = true;
    }
  }
  if !matches!(index, Value::Nil) && !found {
    return Err(args::message("base.next", "invalid key to 'next'"));
  }
  Ok(MultiValue::from_vec(vec![Value::Nil]))
}

fn raw_equal(left: &Value, right: &Value) -> bool {
  match (left, right) {
    (Value::Nil, Value::Nil) => true,
    (Value::Boolean(a), Value::Boolean(b)) => a == b,
    (Value::Integer(a), Value::Integer(b)) => a == b,
    (Value::Integer(a), Value::Number(b)) | (Value::Number(b), Value::Integer(a)) => {
      *a as f64 == *b
    }
    (Value::Number(a), Value::Number(b)) => a == b,
    (Value::String(a), Value::String(b)) => a.as_bytes() == b.as_bytes(),
    (Value::Table(a), Value::Table(b)) => a.to_pointer() == b.to_pointer(),
    (Value::Function(a), Value::Function(b)) => a.to_pointer() == b.to_pointer(),
    (Value::Thread(a), Value::Thread(b)) => a.to_pointer() == b.to_pointer(),
    (Value::UserData(a), Value::UserData(b)) => a.to_pointer() == b.to_pointer(),
    _ => false,
  }
}

/// Parse a Lua-compatible numeric spelling and return its integer or floating-point
/// representation.
pub(super) fn parse_number(text: &str) -> Option<Value> {
  let text = text.trim();
  if let Ok(value) = text.parse::<i64>() {
    Some(Value::Integer(value))
  } else {
    text
      .parse::<f64>()
      .ok()
      .or_else(|| parse_hex_number(text))
      .map(Value::Number)
  }
}

fn parse_hex_number(text: &str) -> Option<f64> {
  let (negative, unsigned) = match text.as_bytes().first()? {
    b'-' => (true, &text[1..]),
    b'+' => (false, &text[1..]),
    _ => (false, text),
  };
  let unsigned = unsigned
    .strip_prefix("0x")
    .or_else(|| unsigned.strip_prefix("0X"))?;
  let (mantissa, exponent) = match unsigned.find(['p', 'P']) {
    Some(position) => {
      if unsigned[position + 1..].contains(['p', 'P']) {
        return None;
      }
      let exponent = &unsigned[position + 1..];
      if exponent.is_empty() || exponent == "+" || exponent == "-" {
        return None;
      }
      let exponent_digits = exponent
        .strip_prefix('+')
        .or_else(|| exponent.strip_prefix('-'))
        .unwrap_or(exponent);
      if !exponent_digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
      }
      let exponent = exponent.parse::<i32>().unwrap_or_else(|_| {
        if exponent.starts_with('-') {
          i32::MIN
        } else {
          i32::MAX
        }
      });
      (&unsigned[..position], exponent)
    }
    None => (unsigned, 0),
  };
  let (integer, fraction) = match mantissa.split_once('.') {
    Some((integer, fraction)) if !fraction.contains('.') => (integer, fraction),
    Some(_) => return None,
    None => (mantissa, ""),
  };
  if integer.is_empty() && fraction.is_empty() {
    return None;
  }

  let mut value = 0.0;
  for digit in integer.chars() {
    value = value * 16.0 + f64::from(digit.to_digit(16)?);
  }
  let mut place = 1.0 / 16.0;
  for digit in fraction.chars() {
    value += f64::from(digit.to_digit(16)?) * place;
    place /= 16.0;
  }
  value *= 2_f64.powi(exponent);
  Some(if negative { -value } else { value })
}
