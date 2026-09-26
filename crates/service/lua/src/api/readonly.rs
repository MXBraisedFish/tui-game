use mlua::{Function, Lua, Table, Value};

pub fn proxy(lua: &Lua, source: Table) -> mlua::Result<Table> {
  let proxy = lua.create_table()?;
  let metatable = lua.create_table()?;
  metatable.set("__index", source.clone())?;
  metatable.raw_set("__tui_game_readonly", true)?;
  metatable.set(
    "__newindex",
    lua.create_function(|_, (_table, _key, _value): (Value, Value, Value)| {
      Err::<(), _>(mlua::Error::RuntimeError(
        "attempt to modify a read-only TUI GAME API table".to_string(),
      ))
    })?,
  )?;
  metatable.set("__len", source.raw_len())?;
  let pairs_source = source.clone();
  metatable.set(
    "__pairs",
    lua.create_function(move |lua, _: Value| iterator(lua, pairs_source.clone(), false))?,
  )?;
  let ipairs_source = source;
  metatable.set(
    "__ipairs",
    lua.create_function(move |lua, _: Value| iterator(lua, ipairs_source.clone(), true))?,
  )?;
  metatable.set("__metatable", false)?;
  proxy.set_metatable(Some(metatable))?;
  Ok(proxy)
}

pub fn backing(table: &Table) -> mlua::Result<Table> {
  if let Some(metatable) = table.metatable()
    && metatable
      .raw_get::<bool>("__tui_game_readonly")
      .unwrap_or(false)
    && let Value::Table(source) = metatable.raw_get::<Value>("__index")?
  {
    return Ok(source);
  }
  Ok(table.clone())
}

pub fn is_proxy(table: &Table) -> mlua::Result<bool> {
  let Some(metatable) = table.metatable() else {
    return Ok(false);
  };
  Ok(
    metatable
      .raw_get::<bool>("__tui_game_readonly")
      .unwrap_or(false),
  )
}

pub fn library(
  lua: &Lua,
  entries: impl IntoIterator<Item = (&'static str, Value)>,
) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, value) in entries {
    source.raw_set(name, value)?;
  }
  proxy(lua, source)
}

pub fn array(lua: &Lua, values: impl IntoIterator<Item = Value>) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (index, value) in values.into_iter().enumerate() {
    source.raw_set(index + 1, value)?;
  }
  proxy(lua, source)
}

pub fn iterator(
  lua: &Lua,
  table: Table,
  array_only: bool,
) -> mlua::Result<(Function, Table, Value)> {
  let state = lua.create_table()?;
  if array_only {
    let source = table;
    let function = lua.create_function(move |_, (_state, index): (Table, i64)| {
      let next = index.saturating_add(1);
      let value = source.raw_get::<Value>(next)?;
      if matches!(value, Value::Nil) {
        Ok((Value::Nil, Value::Nil))
      } else {
        Ok((Value::Integer(next), value))
      }
    })?;
    Ok((function, state, Value::Integer(0)))
  } else {
    let keys = table
      .clone()
      .pairs::<Value, Value>()
      .collect::<mlua::Result<Vec<_>>>()?;
    let index = std::rc::Rc::new(std::cell::Cell::new(0_usize));
    let function = lua.create_function(move |_, _: (Value, Value)| {
      let current = index.get();
      let Some((key, value)) = keys.get(current).cloned() else {
        return Ok((Value::Nil, Value::Nil));
      };
      index.set(current + 1);
      Ok((key, value))
    })?;
    Ok((function, state, Value::Nil))
  }
}
