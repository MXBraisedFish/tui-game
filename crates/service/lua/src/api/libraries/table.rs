use super::*;

use std::collections::{HashMap, HashSet};

pub(super) fn table_lib(lua: &Lua) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  source.raw_set(
    "concat",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::named("table.concat", values, &["table", "sep", "start", "finish"])?;
      let input = mutable_or_readonly_table(
        args::required(&parameters, "table.concat", "table")?,
        "table.concat",
        "table",
      )?;
      let separator = args::optional_string(&parameters, "table.concat", "sep", Some(""))?.unwrap();
      let start = args::optional_integer(&parameters, "table.concat", "start", Some(1))?.unwrap();
      let finish = args::optional_integer(
        &parameters,
        "table.concat",
        "finish",
        Some(input.raw_len() as i64),
      )?
      .unwrap();
      if start < 1 {
        return Err(args::message("table.concat", "start must be at least 1"));
      }
      if finish < start {
        return Ok(String::new());
      }
      checked_entry_count("table.concat", start, finish, args::MAX_API_TABLE_ENTRIES)?;
      let mut parts = Vec::new();
      let mut output_len = 0_usize;
      for index in start..=finish {
        let value = input.raw_get::<Value>(index)?;
        let text = match value {
          Value::String(value) => value.to_str()?.to_string(),
          Value::Integer(value) => value.to_string(),
          Value::Number(value) => value.to_string(),
          value => {
            return Err(args::invalid(
              "table.concat",
              "table",
              "array containing only strings or numbers",
              &value,
            ));
          }
        };
        output_len = output_len
          .checked_add(text.len())
          .and_then(|size| size.checked_add(if parts.is_empty() { 0 } else { separator.len() }))
          .ok_or_else(|| args::message("table.concat", "output size overflow"))?;
        if output_len > args::MAX_API_STRING_BYTES {
          return Err(args::message("table.concat", "output exceeds 1 MiB"));
        }
        parts.push(text);
      }
      Ok(parts.join(&separator))
    })?,
  )?;
  source.raw_set(
    "insert",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::named("table.insert", values, &["table", "position", "value"])?;
      let input = writable_table(
        args::required(&parameters, "table.insert", "table")?,
        "table.insert",
        "table",
      )?;
      let len = input.raw_len();
      if len >= args::MAX_API_TABLE_ENTRIES {
        return Err(args::message(
          "table.insert",
          "array cannot exceed 16384 entries",
        ));
      }
      let position = args::optional_integer(
        &parameters,
        "table.insert",
        "position",
        Some(len as i64 + 1),
      )?
      .unwrap();
      if position < 1 || position > len as i64 + 1 {
        return Err(args::message("table.insert", "position is out of range"));
      }
      let value = args::required(&parameters, "table.insert", "value")?;
      for index in (position as usize..=len).rev() {
        input.raw_set(index + 1, input.raw_get::<Value>(index)?)?;
      }
      input.raw_set(position, value)
    })?,
  )?;
  source.raw_set(
    "move",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::named(
        "table.move",
        values,
        &["source", "start", "finish", "target_index", "target"],
      )?;
      let source = mutable_or_readonly_table(
        args::required(&parameters, "table.move", "source")?,
        "table.move",
        "source",
      )?;
      let target = match parameters.get::<Value>("target")? {
        Value::Nil => writable_table(parameters.get::<Value>("source")?, "table.move", "source")?,
        value => writable_table(value, "table.move", "target")?,
      };
      let start = args::integer(
        args::required(&parameters, "table.move", "start")?,
        "table.move",
        "start",
      )?;
      let finish = args::integer(
        args::required(&parameters, "table.move", "finish")?,
        "table.move",
        "finish",
      )?;
      let target_index = args::integer(
        args::required(&parameters, "table.move", "target_index")?,
        "table.move",
        "target_index",
      )?;
      if finish >= start {
        let count = checked_entry_count("table.move", start, finish, args::MAX_API_TABLE_ENTRIES)?;
        checked_offset("table.move", start, count)?;
        checked_offset("table.move", target_index, count)?;
        let copied = (0..count)
          .map(|offset| {
            let index = start
              .checked_add(offset as i64)
              .ok_or_else(|| args::message("table.move", "source index overflow"))?;
            source.raw_get::<Value>(index)
          })
          .collect::<mlua::Result<Vec<_>>>()?;
        for (offset, value) in copied.into_iter().enumerate() {
          let index = target_index
            .checked_add(offset as i64)
            .ok_or_else(|| args::message("table.move", "target index overflow"))?;
          target.raw_set(index, value)?;
        }
      }
      Ok(target)
    })?,
  )?;
  source.raw_set(
    "pack",
    lua.create_function(|lua, values: MultiValue| {
      let value = args::one("table.pack", "values", values)?;
      let values = args::array_values(value, "table.pack", "values")?;
      let output = lua.create_table()?;
      for (index, value) in values.iter().cloned().enumerate() {
        output.raw_set(index + 1, value)?;
      }
      output.raw_set("n", values.len())?;
      Ok(output)
    })?,
  )?;
  source.raw_set(
    "remove",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::named("table.remove", values, &["table", "position"])?;
      let input = writable_table(
        args::required(&parameters, "table.remove", "table")?,
        "table.remove",
        "table",
      )?;
      let len = input.raw_len();
      if len > args::MAX_API_TABLE_ENTRIES {
        return Err(args::message("table.remove", "array exceeds 16384 entries"));
      }
      if len == 0 {
        return Ok(Value::Nil);
      }
      let position =
        args::optional_integer(&parameters, "table.remove", "position", Some(len as i64))?.unwrap();
      if position < 1 || position > len as i64 {
        return Err(args::message("table.remove", "position is out of range"));
      }
      let removed = input.raw_get::<Value>(position)?;
      for index in position as usize..len {
        input.raw_set(index, input.raw_get::<Value>(index + 1)?)?;
      }
      input.raw_set(len, Value::Nil)?;
      Ok(removed)
    })?,
  )?;
  source.raw_set(
    "sort",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::named("table.sort", values, &["table", "comparator"])?;
      let input = writable_table(
        args::required(&parameters, "table.sort", "table")?,
        "table.sort",
        "table",
      )?;
      let len = input.raw_len();
      if len > 4096 {
        return Err(args::message("table.sort", "array exceeds 4096 items"));
      }
      let comparator = match parameters.get::<Value>("comparator")? {
        Value::Nil => None,
        Value::Function(function) => Some(function),
        value => {
          return Err(args::invalid(
            "table.sort",
            "comparator",
            "function or nil",
            &value,
          ));
        }
      };
      let mut items = (1..=len)
        .map(|index| input.raw_get::<Value>(index))
        .collect::<mlua::Result<Vec<_>>>()?;
      let mut comparison_error = None;
      items.sort_by(|left, right| {
        if comparison_error.is_some() {
          return Ordering::Equal;
        }
        let result = if let Some(comparator) = &comparator {
          comparator.call::<bool>((left.clone(), right.clone()))
        } else {
          default_less(left, right)
        };
        match result {
          Ok(true) => Ordering::Less,
          Ok(false) => match if let Some(comparator) = &comparator {
            comparator.call::<bool>((right.clone(), left.clone()))
          } else {
            default_less(right, left)
          } {
            Ok(true) => Ordering::Greater,
            Ok(false) => Ordering::Equal,
            Err(error) => {
              comparison_error = Some(error);
              Ordering::Equal
            }
          },
          Err(error) => {
            comparison_error = Some(error);
            Ordering::Equal
          }
        }
      });
      if let Some(error) = comparison_error {
        return Err(error);
      }
      for (index, value) in items.into_iter().enumerate() {
        input.raw_set(index + 1, value)?;
      }
      Ok(())
    })?,
  )?;
  source.raw_set(
    "unpack",
    lua.create_function(|_, values: MultiValue| {
      let table = args::named("table.unpack", values, &["table", "start", "finish"])?;
      let value = args::required(&table, "table.unpack", "table")?;
      let Value::Table(input) = value else {
        return Err(args::invalid("table.unpack", "table", "table", &value));
      };
      let input = readonly::backing(&input)?;
      let start = args::optional_integer(&table, "table.unpack", "start", Some(1))?.unwrap();
      let finish = args::optional_integer(
        &table,
        "table.unpack",
        "finish",
        Some(input.raw_len() as i64),
      )?
      .unwrap();
      if start < 1 || finish < start {
        return Ok(MultiValue::new());
      }
      checked_entry_count("table.unpack", start, finish, args::MAX_API_TABLE_ENTRIES)?;
      let mut output = Vec::new();
      for index in start..=finish {
        output.push(input.raw_get(index)?)
      }
      Ok(MultiValue::from_vec(output))
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
      let value = args::one("table.deepcopy", "table", values)?;
      let Value::Table(input) = value else {
        return Err(args::invalid("table.deepcopy", "table", "table", &value));
      };
      let mut copied = HashMap::new();
      let mut entries = 0_usize;
      deep_copy_table(lua, &input, 0, &mut entries, &mut copied)
    })?,
  )?;
  source.raw_set(
    "pretty",
    lua.create_function(|_, values: MultiValue| {
      let value = args::one("table.pretty", "table", values)?;
      let Value::Table(input) = value else {
        return Err(args::invalid("table.pretty", "table", "table", &value));
      };
      let mut writer = PrettyWriter::default();
      let mut entries = 0_usize;
      let mut active = HashSet::new();
      pretty_table(&mut writer, &input, 0, &mut entries, &mut active)?;
      Ok(writer.finish())
    })?,
  )?;
  readonly::proxy(lua, source)
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
  let value = args::one(method, "table", values)?;
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

fn checked_entry_count(method: &str, start: i64, finish: i64, limit: usize) -> mlua::Result<usize> {
  let count = (finish as i128) - (start as i128) + 1;
  let count = usize::try_from(count).map_err(|_| args::message(method, "range is too large"))?;
  if count > limit {
    return Err(args::message(
      method,
      format!("range exceeds {limit} entries"),
    ));
  }
  Ok(count)
}

fn checked_offset(method: &str, start: i64, count: usize) -> mlua::Result<()> {
  if count > 0 {
    start
      .checked_add((count - 1) as i64)
      .ok_or_else(|| args::message(method, "index overflow"))?;
  }
  Ok(())
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

fn default_less(left: &Value, right: &Value) -> mlua::Result<bool> {
  match (left, right) {
    (Value::Integer(left), Value::Integer(right)) => Ok(left < right),
    (Value::Integer(left), Value::Number(right)) => Ok((*left as f64) < *right),
    (Value::Number(left), Value::Integer(right)) => Ok(*left < *right as f64),
    (Value::Number(left), Value::Number(right)) => Ok(left < right),
    (Value::String(left), Value::String(right)) => Ok(left.as_bytes() < right.as_bytes()),
    _ => Err(args::message(
      "table.sort",
      "values are not mutually comparable numbers or strings",
    )),
  }
}
