//! Strict bindings for session-owned and immediate character shaders.

use super::*;
use crate::{LuaCellEffect, LuaEffectRequest, LuaEffectScope};

mod process;
pub(crate) use process::process_buffer;

const OPTIONS: &[&str] = &["process", "scope", "include_continuation"];
const FIELDS: &[&str] = &[
  "width",
  "height",
  "process",
  "scope",
  "include_continuation",
];

/// Register the read-only character shader library.
///
/// # Errors
///
/// Propagate Lua allocation and function registration errors.
pub(super) fn effect(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  source.raw_set("CANVAS", "canvas")?;
  source.raw_set("SLICE", "slice")?;
  let chars_state = state.clone();
  source.raw_set(
    "chars",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "effect.chars";
      let table = positional_table(
        lua,
        method,
        values,
        &["x", "y", "width", "height"],
        &["include_continuation", "scope", "slice_layer"],
      )?;
      let region = tg_service_canvas::CellEffectRegion {
        x: coordinate(&table, method, "x")?,
        y: coordinate(&table, method, "y")?,
        width: positive_u16(&table, method, "width")?,
        height: positive_u16(&table, method, "height")?,
        include_continuation: args::optional_bool(&table, method, "include_continuation", false)?,
      };
      let surface = if scope(&table, method, LuaEffectScope::Canvas)? == LuaEffectScope::Slice {
        Some(match parse_draw_target(&table, method, &chars_state)? {
          LuaDrawTarget::Base => tg_service_canvas::CellSurface::Base,
          LuaDrawTarget::Slice(id) => tg_service_canvas::CellSurface::Slice(id),
        })
      } else {
        None
      };
      let snapshot = chars_state.borrow().completed_frame.clone();
      let result = lua.create_table()?;
      let cells = snapshot.map_or_else(Vec::new, |snapshot| snapshot.cells(surface, region));
      result.raw_set("n", cells.len())?;
      for (index, cell) in cells.iter().enumerate() {
        result.raw_set(index + 1, process::cell_table(lua, cell, (0, 0))?)?;
      }
      Ok(result)
    })?,
  )?;
  let create_state = state.clone();
  source.raw_set(
    "create",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "effect.create";
      let table = positional_table(lua, method, values, &["width", "height"], OPTIONS)?;
      let binding = configuration(lua, &table, method, None)?;
      with_pool_mut(&create_state, method, |pool| {
        if pool.effects.len() >= 1024 {
          return Err(args::message(method, "effect limit of 1024 was reached"));
        }
        pool.next_effect_id = pool
          .next_effect_id
          .checked_add(1)
          .ok_or_else(|| args::message(method, "effect identities exhausted"))?;
        let id = pool.next_effect_id;
        pool.effects.insert(id, binding);
        Ok(format_id(id))
      })
    })?,
  )?;
  let immediate_state = state.clone();
  source.raw_set(
    "effect",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "effect.effect";
      let table = positional_table(
        lua,
        method,
        values,
        &["x", "y", "width", "height"],
        &["process", "scope", "include_continuation", "slice_layer"],
      )?;
      let binding = configuration(lua, &table, method, None)?;
      enqueue(&immediate_state, &table, method, binding)
    })?,
  )?;
  let apply_state = state.clone();
  source.raw_set(
    "apply",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "effect.apply";
      let table = positional_table(lua, method, values, &["id", "x", "y"], &["slice_layer"])?;
      let id = table_id(&table, method)?;
      let binding = with_pool(&apply_state, method, |pool| {
        pool
          .effects
          .get(&id)
          .cloned()
          .ok_or_else(|| args::message(method, "unknown effect id"))
      })?;
      enqueue(&apply_state, &table, method, binding)
    })?,
  )?;
  let set_state = state.clone();
  source.raw_set(
    "set",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "effect.set";
      let table = positional_table(lua, method, values, &["id"], FIELDS)?;
      let id = table_id(&table, method)?;
      with_pool_mut(&set_state, method, |pool| {
        let Some(binding) = pool.effects.get(&id).cloned() else {
          return Ok(false);
        };
        let binding = configuration(lua, &table, method, Some(binding))?;
        pool.effects.insert(id, binding);
        Ok(true)
      })
    })?,
  )?;
  for name in ["delete", "clear", "list", "count", "get_info", "exists"] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let method = format!("effect.{name}");
        let id = if matches!(name, "delete" | "get_info" | "exists") {
          Some(table_id(
            &positional_table(lua, &method, values, &["id"], &[])?,
            &method,
          )?)
        } else {
          args::no_args(&method, values)?;
          None
        };
        with_pool_mut(&state, &method, |pool| match name {
          "delete" => Ok(Value::Boolean(pool.effects.remove(&id.unwrap()).is_some())),
          "exists" => Ok(Value::Boolean(pool.effects.contains_key(&id.unwrap()))),
          "get_info" => pool
            .effects
            .get(&id.unwrap())
            .map(|binding| info(lua, id.unwrap(), binding).map(Value::Table))
            .unwrap_or(Ok(Value::Nil)),
          "clear" => {
            pool.effects.clear();
            Ok(Value::Boolean(true))
          }
          "count" => Ok(Value::Integer(pool.effects.len() as i64)),
          _ => {
            let result = lua.create_table()?;
            for (index, (id, binding)) in pool.effects.iter().enumerate() {
              result.raw_set(index + 1, info(lua, *id, binding)?)?;
            }
            result.raw_set("n", pool.effects.len())?;
            Ok(Value::Table(result))
          }
        })
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}

fn configuration(
  lua: &Lua,
  table: &Table,
  method: &str,
  previous: Option<LuaCellEffect>,
) -> mlua::Result<LuaCellEffect> {
  let mut binding = match previous {
    Some(binding) => binding,
    None => LuaCellEffect {
      width: positive_u16(table, method, "width")?,
      height: positive_u16(table, method, "height")?,
      process: lua.create_function(|_, cell: Table| Ok(cell))?,
      scope: LuaEffectScope::Canvas,
      include_continuation: false,
    },
  };
  for (name, stored) in [
    ("width", &mut binding.width),
    ("height", &mut binding.height),
  ] {
    if !matches!(table.raw_get::<Value>(name)?, Value::Nil) {
      *stored = positive_u16(table, method, name)?;
    }
  }
  match table.raw_get::<Value>("process")? {
    Value::Nil => {}
    Value::Function(function) => binding.process = function,
    value => return Err(args::invalid(method, "process", "function or nil", &value)),
  }
  binding.scope = scope(table, method, binding.scope)?;
  binding.include_continuation = args::optional_bool(
    table,
    method,
    "include_continuation",
    binding.include_continuation,
  )?;
  Ok(binding)
}

fn enqueue(
  state: &SharedApiState,
  table: &Table,
  method: &str,
  effect: LuaCellEffect,
) -> mlua::Result<()> {
  let target = if effect.scope == LuaEffectScope::Slice {
    parse_draw_target(table, method, state)?
  } else {
    LuaDrawTarget::Base
  };
  draw::enqueue_draw(
    state,
    method,
    LuaDrawCommand::Effect(LuaEffectRequest {
      effect,
      target,
      x: coordinate(table, method, "x")?,
      y: coordinate(table, method, "y")?,
    }),
  )
}

fn scope(table: &Table, method: &str, default: LuaEffectScope) -> mlua::Result<LuaEffectScope> {
  match args::optional_string(table, method, "scope", Some(default.as_str()))?
    .unwrap()
    .as_str()
  {
    "canvas" => Ok(LuaEffectScope::Canvas),
    "slice" => Ok(LuaEffectScope::Slice),
    _ => Err(args::message(method, "invalid scope constant")),
  }
}

fn coordinate(table: &Table, method: &str, name: &str) -> mlua::Result<i32> {
  i32::try_from(args::integer(
    args::required(table, method, name)?,
    method,
    name,
  )?)
  .map_err(|_| args::message(method, format!("{name} is out of i32 range")))
}

fn table_id(table: &Table, method: &str) -> mlua::Result<u64> {
  let id = args::string(args::required(table, method, "id")?, method, "id")?;
  id.strip_prefix("effect_")
    .and_then(|id| id.parse::<u64>().ok())
    .filter(|id| *id > 0)
    .ok_or_else(|| args::message(method, "invalid effect id"))
}

fn format_id(id: u64) -> String {
  format!("effect_{id:03}")
}

fn info(lua: &Lua, id: u64, binding: &LuaCellEffect) -> mlua::Result<Table> {
  let result = lua.create_table()?;
  result.raw_set("id", format_id(id))?;
  result.raw_set("width", binding.width)?;
  result.raw_set("height", binding.height)?;
  result.raw_set("process", binding.process.clone())?;
  result.raw_set("scope", binding.scope.as_str())?;
  result.raw_set("include_continuation", binding.include_continuation)?;
  Ok(result)
}
