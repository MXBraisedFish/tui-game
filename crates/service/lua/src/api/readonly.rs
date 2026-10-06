//! Lua proxy tables that reject writes while preserving library lookup and iteration.

use mlua::{Function, Lua, Table, Value};

/// Create a Lua proxy that reads the source table and rejects attempts to assign fields.
///
/// # Errors
///
/// Propagate Lua allocation, table-access, or callback-construction errors while building the
/// read-only table or iterator.
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
  let length_source = source.clone();
  metatable.set(
    "__len",
    lua.create_function(move |_, _table: Value| Ok(length_source.raw_len()))?,
  )?;
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

/// Return a recognized read-only proxy's source table, or the original table for ordinary values.
///
/// # Errors
///
/// Propagate a Lua table-access error while reading a recognized proxy's backing table.
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

/// Report whether the addressed object is proxy.
///
/// # Errors
///
/// This marker check returns `Ok(false)` when no recognized proxy metatable exists; marker-read
/// failures also resolve to false.
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

/// Build a named Lua library table and expose it through a read-only proxy.
///
/// # Errors
///
/// Propagate Lua allocation, table-access, or callback-construction errors while building the
/// read-only table or iterator.
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

/// Build a one-based Lua array and expose it through a read-only proxy.
///
/// # Errors
///
/// Propagate Lua allocation, table-access, or callback-construction errors while building the
/// read-only table or iterator.
pub fn array(lua: &Lua, values: impl IntoIterator<Item = Value>) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (index, value) in values.into_iter().enumerate() {
    source.raw_set(index + 1, value)?;
  }
  proxy(lua, source)
}

/// Return the iterator, state, and initial control value for a table or contiguous-array
/// traversal.
///
/// # Arguments
///
/// * `lua` - The Lua VM in which values and callbacks are created.
/// * `table` - The Lua table to inspect or convert.
/// * `array_only` - The array only.
///
/// # Errors
///
/// Propagate Lua allocation, table-access, or callback-construction errors while building the
/// read-only table or iterator.
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
