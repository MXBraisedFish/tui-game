use super::*;

use std::collections::{HashMap, HashSet};

pub(super) fn table_lib(lua: &Lua) -> mlua::Result<Table> {
  let source = lua.globals().get::<Table>("table")?;
  let length = lua
    .load("function(value) return #value end")
    .eval::<Function>()?;

  let concat = source.get::<Function>("concat")?;
  source.raw_set("concat", bounded_concat(lua, concat, length.clone())?)?;
  let insert = source.get::<Function>("insert")?;
  let insert_length = length.clone();
  source.raw_set(
    "insert",
    bounded_standard_function(lua, insert, move |arguments| {
      reject_readonly_table(arguments, 0, "table.insert")?;
      if let Some(table) = table_argument_at(arguments, 0) {
        let length = checked_table_length(&insert_length, &table, "table.insert")?;
        if length >= args::MAX_API_TABLE_ENTRIES as i64 {
          return Err(args::message(
            "table.insert",
            "array cannot exceed 16384 entries",
          ));
        }
      }
      Ok(())
    })?,
  )?;
  let move_table = source.get::<Function>("move")?;
  source.raw_set(
    "move",
    bounded_standard_function(lua, move_table, |arguments| {
      let source_table = table_argument_at(arguments, 0);
      let target_value = arguments.get(4).cloned().unwrap_or(Value::Nil);
      let target_table = match target_value {
        Value::Nil => source_table.clone(),
        Value::Table(table) => Some(table),
        _ => None,
      };
      if let Some(target) = target_table
        && readonly::is_proxy(&target)?
      {
        return Err(readonly_write_error());
      }
      if let (Some(start), Some(finish)) = (
        integer_argument(arguments, 1),
        integer_argument(arguments, 2),
      ) {
        reject_large_span("table.move", start, finish, args::MAX_API_TABLE_ENTRIES)?;
      }
      Ok(())
    })?,
  )?;
  let pack = source.get::<Function>("pack")?;
  source.raw_set(
    "pack",
    bounded_standard_function(lua, pack, |arguments| {
      if arguments.len() > args::MAX_API_TABLE_ENTRIES {
        return Err(args::message(
          "table.pack",
          "arguments exceed 16384 entries",
        ));
      }
      Ok(())
    })?,
  )?;
  let remove = source.get::<Function>("remove")?;
  let remove_length = length.clone();
  source.raw_set(
    "remove",
    bounded_standard_function(lua, remove, move |arguments| {
      reject_readonly_table(arguments, 0, "table.remove")?;
      if let Some(table) = table_argument_at(arguments, 0) {
        let length = checked_table_length(&remove_length, &table, "table.remove")?;
        if length > args::MAX_API_TABLE_ENTRIES as i64 {
          return Err(args::message("table.remove", "array exceeds 16384 entries"));
        }
      }
      Ok(())
    })?,
  )?;
  let sort = source.get::<Function>("sort")?;
  let sort_length = length.clone();
  source.raw_set(
    "sort",
    bounded_standard_function(lua, sort, move |arguments| {
      reject_readonly_table(arguments, 0, "table.sort")?;
      if let Some(table) = table_argument_at(arguments, 0) {
        let length = checked_table_length(&sort_length, &table, "table.sort")?;
        if length > 4096 {
          return Err(args::message("table.sort", "array exceeds 4096 items"));
        }
      }
      Ok(())
    })?,
  )?;
  let unpack = source.get::<Function>("unpack")?;
  source.raw_set(
    "unpack",
    bounded_standard_function(lua, unpack, move |arguments| {
      if let Some(table) = table_argument_at(arguments, 0) {
        let start = match optional_integer_argument(arguments, 1) {
          Some(start) => start,
          None
            if arguments
              .get(1)
              .is_none_or(|value| matches!(value, Value::Nil)) =>
          {
            1
          }
          None => return Ok(()),
        };
        let finish = match optional_integer_argument(arguments, 2) {
          Some(finish) => finish,
          None
            if arguments
              .get(2)
              .is_none_or(|value| matches!(value, Value::Nil)) =>
          {
            checked_table_length(&length, &table, "table.unpack")?
          }
          None => return Ok(()),
        };
        reject_large_span("table.unpack", start, finish, args::MAX_API_TABLE_ENTRIES)?;
      }
      Ok(())
    })?,
  )?;
  source.raw_set(
    "count",
    lua.create_function(|lua, values: MultiValue| {
      let input = table_argument("table.count", values, false)?;
      let shape = inspect_table_shape("table.count", &input)?;
      let output = lua.create_table()?;
      output.raw_set("n", shape.total_count())?;
      output.raw_set("contiguous", shape.is_contiguous())?;
      Ok(output)
    })?,
  )?;
  source.raw_set(
    "count_array",
    lua.create_function(|lua, values: MultiValue| {
      let input = table_argument("table.count_array", values, false)?;
      let shape = inspect_table_shape("table.count_array", &input)?;
      let indexes = lua.create_table()?;
      for (position, index) in shape.array_indexes.iter().copied().enumerate() {
        indexes.raw_set(position + 1, index)?;
      }
      let output = lua.create_table()?;
      output.raw_set("n", shape.array_indexes.len())?;
      output.raw_set("contiguous", shape.is_contiguous())?;
      output.raw_set("indexes", indexes)?;
      Ok(output)
    })?,
  )?;
  source.raw_set(
    "count_hash",
    lua.create_function(|_, values: MultiValue| {
      let input = table_argument("table.count_hash", values, false)?;
      let shape = inspect_table_shape("table.count_hash", &input)?;
      Ok(shape.hash_count)
    })?,
  )?;
  source.raw_set(
    "compact",
    lua.create_function(|_, values: MultiValue| {
      let input = table_argument("table.compact", values, true)?;
      let mut array_entries = Vec::new();
      for pair in input.clone().pairs::<Value, Value>() {
        let (key, value) = pair?;
        if let Value::Integer(index) = key
          && index >= 1
        {
          array_entries.push((index, value));
          if array_entries.len() > args::MAX_API_TABLE_ENTRIES {
            return Err(args::message(
              "table.compact",
              "table exceeds 16384 entries",
            ));
          }
        }
      }
      array_entries.sort_by_key(|(index, _)| *index);
      for (index, _) in &array_entries {
        input.raw_set(*index, Value::Nil)?;
      }
      for (position, (_, value)) in array_entries.into_iter().enumerate() {
        input.raw_set(position + 1, value)?;
      }
      Ok(input)
    })?,
  )?;
  source.raw_set(
    "deepcopy",
    lua.create_function(|lua, values: MultiValue| {
      let input = table_argument("table.deepcopy", values, false)?;
      let mut copied = HashMap::new();
      let mut entries = 0_usize;
      deep_copy_table(lua, &input, 0, &mut entries, &mut copied)
    })?,
  )?;
  source.raw_set(
    "pretty",
    lua.create_function(|_, values: MultiValue| {
      let input = table_argument("table.pretty", values, false)?;
      let mut writer = PrettyWriter::default();
      let mut entries = 0_usize;
      let mut active = HashSet::new();
      pretty_table(&mut writer, &input, 0, &mut entries, &mut active)?;
      Ok(writer.finish())
    })?,
  )?;
  readonly::proxy(lua, source)
}

fn bounded_standard_function<F>(lua: &Lua, function: Function, check: F) -> mlua::Result<Function>
where
  F: Fn(&MultiValue) -> mlua::Result<()> + 'static,
{
  lua.create_function(move |_, arguments: MultiValue| {
    check(&arguments)?;
    function.call::<MultiValue>(arguments)
  })
}

fn bounded_concat(lua: &Lua, function: Function, length: Function) -> mlua::Result<Function> {
  lua.create_function(move |_, arguments: MultiValue| {
    if let Some(table) = table_argument_at(&arguments, 0) {
      let start = match optional_integer_argument(&arguments, 2) {
        Some(start) => start,
        None
          if arguments
            .get(2)
            .is_none_or(|value| matches!(value, Value::Nil)) =>
        {
          1
        }
        None => return function.call::<MultiValue>(arguments),
      };
      let finish = match optional_integer_argument(&arguments, 3) {
        Some(finish) => finish,
        None
          if arguments
            .get(3)
            .is_none_or(|value| matches!(value, Value::Nil)) =>
        {
          checked_table_length(&length, &table, "table.concat")?
        }
        None => return function.call::<MultiValue>(arguments),
      };
      reject_large_span("table.concat", start, finish, args::MAX_API_TABLE_ENTRIES)?;
    }
    let results = function.call::<MultiValue>(arguments)?;
    if let Some(Value::String(value)) = results.front()
      && value.as_bytes().len() > args::MAX_API_STRING_BYTES
    {
      return Err(args::message("table.concat", "output exceeds 1 MiB"));
    }
    Ok(results)
  })
}

fn table_argument_at(arguments: &MultiValue, index: usize) -> Option<Table> {
  match arguments.get(index) {
    Some(Value::Table(table)) => Some(table.clone()),
    _ => None,
  }
}

fn checked_table_length(length: &Function, table: &Table, method: &str) -> mlua::Result<i64> {
  let value = length.call::<Value>(table.clone())?;
  args::integer(value, method, "table length")
}

fn optional_integer_argument(arguments: &MultiValue, index: usize) -> Option<i64> {
  let value = arguments.get(index)?.clone();
  if matches!(value, Value::Nil) {
    return None;
  }
  integer_argument(arguments, index)
}

fn integer_argument(arguments: &MultiValue, index: usize) -> Option<i64> {
  match arguments.get(index)? {
    Value::Integer(value) => Some(*value),
    Value::Number(value) if value.is_finite() && value.fract() == 0.0 => {
      if *value < i64::MIN as f64 || *value >= 9_223_372_036_854_775_808.0 {
        None
      } else {
        Some(*value as i64)
      }
    }
    Value::String(value) => {
      let text = value.to_str().ok()?;
      match super::base::parse_number(text.as_ref())? {
        Value::Integer(value) => Some(value),
        Value::Number(value)
          if value.is_finite()
            && value.fract() == 0.0
            && value >= i64::MIN as f64
            && value < 9_223_372_036_854_775_808.0 =>
        {
          Some(value as i64)
        }
        _ => None,
      }
    }
    _ => None,
  }
}

fn reject_large_span(method: &str, start: i64, finish: i64, limit: usize) -> mlua::Result<()> {
  if finish >= start && (finish as i128 - start as i128 + 1) > limit as i128 {
    return Err(args::message(
      method,
      format!("range exceeds {limit} entries"),
    ));
  }
  Ok(())
}

fn reject_readonly_table(arguments: &MultiValue, index: usize, method: &str) -> mlua::Result<()> {
  if let Some(table) = table_argument_at(arguments, index)
    && readonly::is_proxy(&table)?
  {
    return Err(args::message(method, "parameter 'table' is read-only"));
  }
  Ok(())
}

fn readonly_write_error() -> mlua::Error {
  mlua::Error::RuntimeError("attempt to modify a read-only TUI GAME API table".to_string())
}

struct TableShape {
  array_indexes: Vec<i64>,
  hash_count: usize,
}

impl TableShape {
  fn total_count(&self) -> usize {
    self.array_indexes.len().saturating_add(self.hash_count)
  }

  fn is_contiguous(&self) -> bool {
    self
      .array_indexes
      .iter()
      .copied()
      .enumerate()
      .all(|(position, index)| index == position as i64 + 1)
  }
}

fn table_argument(method: &str, values: MultiValue, writable: bool) -> mlua::Result<Table> {
  let parameters = args::named(method, values, &["table"])?;
  let value = args::required(&parameters, method, "table")?;
  if writable {
    writable_table(value, method, "table")
  } else {
    mutable_or_readonly_table(value, method, "table")
  }
}

fn inspect_table_shape(method: &str, input: &Table) -> mlua::Result<TableShape> {
  let mut array_indexes = Vec::new();
  let mut hash_count = 0_usize;
  let mut total_count = 0_usize;
  for pair in input.clone().pairs::<Value, Value>() {
    let (key, _) = pair?;
    total_count = total_count.saturating_add(1);
    if total_count > args::MAX_API_TABLE_ENTRIES {
      return Err(args::message(method, "table exceeds 16384 entries"));
    }
    if let Value::Integer(index) = key
      && index >= 1
    {
      array_indexes.push(index);
    } else {
      hash_count = hash_count.saturating_add(1);
    }
  }
  array_indexes.sort_unstable();
  Ok(TableShape {
    array_indexes,
    hash_count,
  })
}

fn deep_copy_table(
  lua: &Lua,
  input: &Table,
  depth: usize,
  entries: &mut usize,
  copied: &mut HashMap<usize, Table>,
) -> mlua::Result<Table> {
  if depth >= 32 {
    return Err(args::message("table.deepcopy", "table exceeds 32 levels"));
  }
  let input = readonly::backing(input)?;
  let pointer = input.to_pointer() as usize;
  if let Some(existing) = copied.get(&pointer) {
    return Ok(existing.clone());
  }
  let output = lua.create_table()?;
  copied.insert(pointer, output.clone());
  for pair in input.pairs::<Value, Value>() {
    let (key, value) = pair?;
    *entries = entries.saturating_add(1);
    if *entries > args::MAX_API_TABLE_ENTRIES {
      return Err(args::message(
        "table.deepcopy",
        "table exceeds 16384 entries",
      ));
    }
    let key = deep_copy_value(lua, key, depth + 1, entries, copied)?;
    let value = deep_copy_value(lua, value, depth + 1, entries, copied)?;
    output.raw_set(key, value)?;
  }
  Ok(output)
}

fn deep_copy_value(
  lua: &Lua,
  value: Value,
  depth: usize,
  entries: &mut usize,
  copied: &mut HashMap<usize, Table>,
) -> mlua::Result<Value> {
  match value {
    Value::Table(table) => deep_copy_table(lua, &table, depth, entries, copied).map(Value::Table),
    value => Ok(value),
  }
}

#[derive(Default)]
struct PrettyWriter {
  output: String,
}

impl PrettyWriter {
  fn push(&mut self, value: &str) -> mlua::Result<()> {
    let next_len = self
      .output
      .len()
      .checked_add(value.len())
      .ok_or_else(|| args::message("table.pretty", "output size overflow"))?;
    if next_len > args::MAX_API_STRING_BYTES {
      return Err(args::message("table.pretty", "output exceeds 1 MiB"));
    }
    self.output.push_str(value);
    Ok(())
  }

  fn finish(self) -> String {
    self.output
  }
}

fn pretty_table(
  writer: &mut PrettyWriter,
  input: &Table,
  depth: usize,
  entries: &mut usize,
  active: &mut HashSet<usize>,
) -> mlua::Result<()> {
  if depth >= 32 {
    return Err(args::message("table.pretty", "table exceeds 32 levels"));
  }
  let input = readonly::backing(input)?;
  let pointer = input.to_pointer() as usize;
  if !active.insert(pointer) {
    return writer.push("<cycle>");
  }
  let mut values = input
    .pairs::<Value, Value>()
    .collect::<mlua::Result<Vec<_>>>()?;
  *entries = entries.saturating_add(values.len());
  if *entries > args::MAX_API_TABLE_ENTRIES {
    active.remove(&pointer);
    return Err(args::message("table.pretty", "table exceeds 16384 entries"));
  }
  values.sort_by(|(left, _), (right, _)| pretty_key_order(left, right));

  writer.push("{")?;
  for (index, (key, value)) in values.into_iter().enumerate() {
    if index > 0 {
      writer.push(", ")?;
    }
    pretty_key(writer, &key)?;
    writer.push(" = ")?;
    pretty_value(writer, &value, depth + 1, entries, active)?;
  }
  writer.push("}")?;
  active.remove(&pointer);
  Ok(())
}

fn pretty_value(
  writer: &mut PrettyWriter,
  value: &Value,
  depth: usize,
  entries: &mut usize,
  active: &mut HashSet<usize>,
) -> mlua::Result<()> {
  match value {
    Value::Nil => writer.push("nil"),
    Value::Boolean(value) => writer.push(if *value { "true" } else { "false" }),
    Value::Integer(value) => writer.push(&value.to_string()),
    Value::Number(value) => writer.push(&value.to_string()),
    Value::String(value) => pretty_string(writer, value),
    Value::Table(value) => pretty_table(writer, value, depth, entries, active),
    Value::Function(value) => writer.push(&format!("<function:{:p}>", value.to_pointer())),
    Value::Thread(value) => writer.push(&format!("<thread:{:p}>", value.to_pointer())),
    Value::UserData(value) => writer.push(&format!("<userdata:{:p}>", value.to_pointer())),
    Value::LightUserData(value) => writer.push(&format!("<lightuserdata:{:p}>", value.0)),
    Value::Error(_) => writer.push("<error>"),
    Value::Other(_) => writer.push("<other>"),
  }
}

fn pretty_key(writer: &mut PrettyWriter, key: &Value) -> mlua::Result<()> {
  if let Value::String(value) = key
    && let Ok(text) = value.to_str()
    && is_identifier(&text)
  {
    return writer.push(&text);
  }
  writer.push("[")?;
  pretty_scalar(writer, key)?;
  writer.push("]")
}

fn pretty_scalar(writer: &mut PrettyWriter, value: &Value) -> mlua::Result<()> {
  match value {
    Value::Boolean(value) => writer.push(if *value { "true" } else { "false" }),
    Value::Integer(value) => writer.push(&value.to_string()),
    Value::Number(value) => writer.push(&value.to_string()),
    Value::String(value) => pretty_string(writer, value),
    Value::Table(value) => writer.push(&format!("<table:{:p}>", value.to_pointer())),
    Value::Function(value) => writer.push(&format!("<function:{:p}>", value.to_pointer())),
    Value::Thread(value) => writer.push(&format!("<thread:{:p}>", value.to_pointer())),
    Value::UserData(value) => writer.push(&format!("<userdata:{:p}>", value.to_pointer())),
    Value::LightUserData(value) => writer.push(&format!("<lightuserdata:{:p}>", value.0)),
    Value::Error(_) => writer.push("<error>"),
    Value::Other(_) => writer.push("<other>"),
    Value::Nil => writer.push("nil"),
  }
}

fn pretty_string(writer: &mut PrettyWriter, value: &mlua::LuaString) -> mlua::Result<()> {
  writer.push("\"")?;
  match value.to_str() {
    Ok(value) => {
      for character in value.chars() {
        match character {
          '\\' => writer.push("\\\\")?,
          '"' => writer.push("\\\"")?,
          '\n' => writer.push("\\n")?,
          '\r' => writer.push("\\r")?,
          '\t' => writer.push("\\t")?,
          '\0' => writer.push("\\0")?,
          character if character.is_control() => {
            writer.push(&format!("\\u{{{:x}}}", character as u32))?;
          }
          character => writer.push(character.encode_utf8(&mut [0; 4]))?,
        }
      }
    }
    Err(_) => {
      for byte in value.as_bytes().iter() {
        writer.push(&format!("\\x{byte:02X}"))?;
      }
    }
  }
  writer.push("\"")
}

fn is_identifier(value: &str) -> bool {
  let mut characters = value.chars();
  let Some(first) = characters.next() else {
    return false;
  };
  (first == '_' || first.is_ascii_alphabetic())
    && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn pretty_key_order(left: &Value, right: &Value) -> Ordering {
  pretty_key_rank(left)
    .cmp(&pretty_key_rank(right))
    .then_with(|| match (left, right) {
      (Value::Integer(left), Value::Integer(right)) => left.cmp(right),
      (Value::Integer(left), Value::Number(right)) => (*left as f64).total_cmp(right),
      (Value::Number(left), Value::Integer(right)) => left.total_cmp(&(*right as f64)),
      (Value::Number(left), Value::Number(right)) => left.total_cmp(right),
      (Value::String(left), Value::String(right)) => {
        left.as_bytes().as_ref().cmp(right.as_bytes().as_ref())
      }
      (Value::Boolean(left), Value::Boolean(right)) => left.cmp(right),
      _ => pretty_identity(left).cmp(&pretty_identity(right)),
    })
}

fn pretty_key_rank(value: &Value) -> u8 {
  match value {
    Value::Integer(_) | Value::Number(_) => 0,
    Value::String(_) => 1,
    Value::Boolean(_) => 2,
    Value::Table(_) => 3,
    Value::Function(_) => 4,
    Value::Thread(_) => 5,
    Value::UserData(_) => 6,
    Value::LightUserData(_) => 7,
    Value::Error(_) => 8,
    Value::Other(_) => 9,
    Value::Nil => 10,
  }
}

fn pretty_identity(value: &Value) -> usize {
  match value {
    Value::Table(value) => value.to_pointer() as usize,
    Value::Function(value) => value.to_pointer() as usize,
    Value::Thread(value) => value.to_pointer() as usize,
    Value::UserData(value) => value.to_pointer() as usize,
    Value::LightUserData(value) => value.0 as usize,
    _ => 0,
  }
}

fn mutable_or_readonly_table(value: Value, method: &str, name: &str) -> mlua::Result<Table> {
  let Value::Table(table) = value else {
    return Err(args::invalid(method, name, "table", &value));
  };
  readonly::backing(&table)
}

fn writable_table(value: Value, method: &str, name: &str) -> mlua::Result<Table> {
  let Value::Table(table) = value else {
    return Err(args::invalid(method, name, "table", &value));
  };
  if readonly::is_proxy(&table)? {
    return Err(args::message(
      method,
      format!("parameter '{name}' is read-only"),
    ));
  }
  Ok(table)
}
