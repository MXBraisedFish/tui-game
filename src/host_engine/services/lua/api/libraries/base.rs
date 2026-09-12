use super::*;

pub(super) fn base(lua: &Lua) -> mlua::Result<Table> {
  let ipairs = lua.create_function(|lua, args: MultiValue| {
    let value = args::one("base.ipairs", "table", args)?;
    let Value::Table(table) = value else {
      return Err(args::invalid("base.ipairs", "table", "table", &value));
    };
    record_iterator(lua, readonly::backing(&table)?, true)
  })?;
  let pairs = lua.create_function(|lua, args: MultiValue| {
    let value = args::one("base.pairs", "table", args)?;
    let Value::Table(table) = value else {
      return Err(args::invalid("base.pairs", "table", "table", &value));
    };
    pairs_iterator(lua, table)
  })?;
  let next = lua.create_function(|lua, args: MultiValue| {
    let table = args::named("base.next", args, &["table", "index"])?;
    let value = args::required(&table, "base.next", "table")?;
    let Value::Table(source) = value else {
      return Err(args::invalid("base.next", "table", "table", &value));
    };
    let source = readonly::backing(&source)?;
    let index = table.get::<Value>("index")?;
    let mut found = matches!(index, Value::Nil);
    for pair in source.pairs::<Value, Value>() {
      let (key, value) = pair?;
      if found {
        return iteration_record(lua, key, value).map(Value::Table);
      }
      if raw_equal(&key, &index) {
        found = true;
      }
    }
    Ok(Value::Nil)
  })?;
  let select = lua.create_function(|_, args: MultiValue| {
    let table = args::named("base.select", args, &["index", "values"])?;
    let index = args::required(&table, "base.select", "index")?;
    let values = args::values(&table, "base.select")?;
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
    let table = args::named("base.rawequal", args, &["left", "right"])?;
    Ok(raw_equal(
      &table.get::<Value>("left")?,
      &table.get::<Value>("right")?,
    ))
  })?;
  let rawlen = lua.create_function(|_, args: MultiValue| {
    let value = args::one("base.rawlen", "value", args)?;
    match value {
      Value::String(value) => Ok(value.as_bytes().len() as i64),
      Value::Table(table) => Ok(readonly::backing(&table)?.raw_len() as i64),
      value => Err(args::invalid(
        "base.rawlen",
        "value",
        "string or table",
        &value,
      )),
    }
  })?;
  let tonumber = lua.create_function(|_, args: MultiValue| {
    let table = args::named("base.tonumber", args, &["value", "base"])?;
    let value = args::required(&table, "base.tonumber", "value")?;
    let base = args::optional_integer(&table, "base.tonumber", "base", None)?;
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
      (value, Some(_)) => Err(args::invalid(
        "base.tonumber",
        "base",
        "integer 2..36",
        &value,
      )),
      _ => Ok(Value::Nil),
    }
  })?;
  let tostring = lua.create_function(|lua, args: MultiValue| {
    let value = args::one("base.tostring", "value", args)?;
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
    let value = args::one("base.type", "value", args)?;
    let type_name = match value {
      Value::Integer(_) | Value::Number(_) => "number",
      _ => args::type_name(&value),
    };
    lua.create_string(type_name)
  })?;
  let setmetatable = lua.create_function(|_, args: MultiValue| {
    let parameters = args::named("base.setmetatable", args, &["table", "metatable"])?;
    let target = parameters.get::<Value>("table")?;
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
    let metatable = match parameters.get::<Value>("metatable")? {
      Value::Nil => None,
      Value::Table(metatable) => Some(metatable),
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
    let value = args::one("base.getmetatable", "table", args)?;
    let Value::Table(target) = value else {
      return Err(args::invalid("base.getmetatable", "table", "table", &value));
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
      ("rawlen", function_value(rawlen)),
      ("tonumber", function_value(tonumber)),
      ("tostring", function_value(tostring)),
      ("type", function_value(type_fn)),
      ("setmetatable", function_value(setmetatable)),
      ("getmetatable", function_value(getmetatable)),
    ],
  )
}

fn record_iterator(lua: &Lua, table: Table, array_only: bool) -> mlua::Result<Function> {
  if array_only {
    let index = std::rc::Rc::new(std::cell::Cell::new(0_i64));
    lua.create_function(move |lua, _: MultiValue| {
      let next = index.get().saturating_add(1);
      let value = table.get::<Value>(next)?;
      if matches!(value, Value::Nil) {
        Ok(Value::Nil)
      } else {
        index.set(next);
        iteration_record(lua, Value::Integer(next), value).map(Value::Table)
      }
    })
  } else {
    let values = table
      .pairs::<Value, Value>()
      .collect::<mlua::Result<Vec<_>>>()?;
    let index = std::rc::Rc::new(std::cell::Cell::new(0_usize));
    lua.create_function(move |lua, _: MultiValue| {
      let current = index.get();
      let Some((key, value)) = values.get(current).cloned() else {
        return Ok(Value::Nil);
      };
      index.set(current + 1);
      iteration_record(lua, key, value).map(Value::Table)
    })
  }
}

fn pairs_iterator(lua: &Lua, table: Table) -> mlua::Result<Function> {
  if readonly::is_proxy(&table)? {
    return record_iterator(lua, readonly::backing(&table)?, false);
  }
  if let Some(metatable) = table.metatable() {
    let metamethod = metatable.raw_get::<Value>("__pairs")?;
    if !matches!(metamethod, Value::Nil) {
      let Value::Function(metamethod) = metamethod else {
        return Err(args::invalid(
          "base.pairs",
          "__pairs",
          "function",
          &metamethod,
        ));
      };
      let mut results = metamethod.call::<MultiValue>(table)?;
      let iterator = results.pop_front().unwrap_or(Value::Nil);
      let Value::Function(iterator) = iterator else {
        return Err(args::invalid(
          "base.pairs",
          "__pairs iterator",
          "function",
          &iterator,
        ));
      };
      let state = results.pop_front().unwrap_or(Value::Nil);
      let initial = results.pop_front().unwrap_or(Value::Nil);
      let control = std::rc::Rc::new(std::cell::RefCell::new(initial));
      return lua.create_function(move |lua, _: MultiValue| {
        let current = control.borrow().clone();
        let mut values = iterator.call::<MultiValue>((state.clone(), current))?;
        let index = values.pop_front().unwrap_or(Value::Nil);
        if matches!(index, Value::Nil) {
          return Ok(Value::Nil);
        }
        let value = values.pop_front().unwrap_or(Value::Nil);
        *control.borrow_mut() = index.clone();
        iteration_record(lua, index, value).map(Value::Table)
      });
    }
  }
  record_iterator(lua, table, false)
}

fn iteration_record(lua: &Lua, index: Value, value: Value) -> mlua::Result<Table> {
  let result = lua.create_table()?;
  result.set("index", index)?;
  result.set("value", value)?;
  Ok(result)
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

fn parse_number(text: &str) -> Option<Value> {
  let text = text.trim();
  if let Ok(value) = text.parse::<i64>() {
    Some(Value::Integer(value))
  } else {
    text.parse::<f64>().ok().map(Value::Number)
  }
}
