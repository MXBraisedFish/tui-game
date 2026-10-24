//! Strict Lua bindings for session-owned transparent pointer regions.

use super::*;
use crate::object_pool::LuaHitboxBinding;
use tg_service_widget::{HitAreaId, HitAreaOptions, HitAreaService};

const MAX_HITBOXES: usize = 1024;

/// Register the hitbox methods in a read-only library table.
///
/// # Errors
///
/// Propagate Lua allocation or function registration errors.
pub(super) fn hitbox(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  let create_state = state.clone();
  source.raw_set(
    "create",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.hitbox.create";
      let table = positional_table(
        lua,
        method,
        values,
        &["width", "height"],
        &["hover_move", "drag"],
      )?;
      let binding = options(
        &table,
        method,
        LuaHitboxBinding {
          width: positive_u16(&table, method, "width")?,
          height: positive_u16(&table, method, "height")?,
          options: HitAreaOptions::default(),
          placement: None,
        },
      )?;
      with_pool_mut(&create_state, method, |pool| {
        if pool.hitboxes.len() >= MAX_HITBOXES {
          return Err(args::message(method, "hitbox limit of 1024 was reached"));
        }
        let id = HitAreaService::new().create(pool.ui_mut(), binding.options);
        pool.hitboxes.insert(id.0, binding);
        Ok(format_id(id.0))
      })
    })?,
  )?;

  let set_state = state.clone();
  source.raw_set(
    "set",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.hitbox.set";
      let table = positional_table(
        lua,
        method,
        values,
        &["id"],
        &["width", "height", "hover_move", "drag"],
      )?;
      let id = table_id(&table, method)?;
      with_pool_mut(&set_state, method, |pool| {
        let Some(binding) = pool.hitboxes.get(&id).copied() else {
          return Ok(false);
        };
        let binding = options(&table, method, binding)?;
        HitAreaService::new().set_options(pool.ui_mut(), HitAreaId(id), binding.options);
        pool.hitboxes.insert(id, binding);
        Ok(true)
      })
    })?,
  )?;

  let draw_state = state.clone();
  source.raw_set(
    "draw",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.hitbox.draw";
      let table = positional_table(lua, method, values, &["id", "x", "y"], &["slice_layer"])?;
      let id = table_id(&table, method)?;
      let coordinate = |name| {
        let value = args::integer(args::required(&table, method, name)?, method, name)?;
        i32::try_from(value)
          .map_err(|_| args::message(method, format!("{name} is out of i32 range")))
      };
      let command = LuaDrawCommand::Hitbox {
        target: parse_draw_target(&table, method, &draw_state)?,
        id,
        x: coordinate("x")?,
        y: coordinate("y")?,
      };
      with_pool(&draw_state, method, |pool| {
        if !pool.hitboxes.contains_key(&id) {
          return Err(args::message(method, "unknown hitbox id"));
        }
        Ok(())
      })?;
      draw::enqueue_draw(&draw_state, method, command)
    })?,
  )?;

  for (name, method) in [
    ("delete", "widget.hitbox.delete"),
    ("exists", "widget.hitbox.exists"),
    ("get_info", "widget.hitbox.get_info"),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let table = positional_table(lua, method, values, &["id"], &[])?;
        let id = table_id(&table, method)?;
        with_pool_mut(&state, method, |pool| match name {
          "delete" => {
            let removed = pool.hitboxes.remove(&id).is_some();
            if removed {
              HitAreaService::new().remove(pool.ui_mut(), HitAreaId(id));
            }
            Ok(Value::Boolean(removed))
          }
          "exists" => Ok(Value::Boolean(pool.hitboxes.contains_key(&id))),
          _ => pool
            .hitboxes
            .get(&id)
            .map(|binding| info(lua, id, binding).map(Value::Table))
            .unwrap_or(Ok(Value::Nil)),
        })
      })?,
    )?;
  }
  for (name, method) in [
    ("clear", "widget.hitbox.clear"),
    ("count", "widget.hitbox.count"),
    ("list", "widget.hitbox.list"),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        args::no_args(method, values)?;
        with_pool_mut(&state, method, |pool| match name {
          "clear" => {
            for id in std::mem::take(&mut pool.hitboxes).into_keys() {
              HitAreaService::new().remove(pool.ui_mut(), HitAreaId(id));
            }
            Ok(Value::Boolean(true))
          }
          "count" => Ok(Value::Integer(pool.hitboxes.len() as i64)),
          _ => {
            let result = lua.create_table()?;
            for (index, (id, binding)) in pool.hitboxes.iter().enumerate() {
              result.raw_set(index + 1, info(lua, *id, binding)?)?;
            }
            result.raw_set("n", pool.hitboxes.len())?;
            Ok(Value::Table(result))
          }
        })
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}

fn options(
  table: &Table,
  method: &str,
  mut binding: LuaHitboxBinding,
) -> mlua::Result<LuaHitboxBinding> {
  if !matches!(table.raw_get::<Value>("width")?, Value::Nil) {
    binding.width = positive_u16(table, method, "width")?;
  }
  if !matches!(table.raw_get::<Value>("height")?, Value::Nil) {
    binding.height = positive_u16(table, method, "height")?;
  }
  binding.options.hover_move =
    args::optional_bool(table, method, "hover_move", binding.options.hover_move)?;
  binding.options.drag = args::optional_bool(table, method, "drag", binding.options.drag)?;
  Ok(binding)
}

fn table_id(table: &Table, method: &str) -> mlua::Result<u64> {
  let value = args::string(args::required(table, method, "id")?, method, "id")?;
  value
    .strip_prefix("hitbox_")
    .and_then(|value| value.parse().ok())
    .filter(|id| *id > 0)
    .ok_or_else(|| args::message(method, "invalid hitbox id"))
}

fn format_id(id: u64) -> String {
  format!("hitbox_{id:03}")
}

fn info(lua: &Lua, id: u64, binding: &LuaHitboxBinding) -> mlua::Result<Table> {
  let result = lua.create_table()?;
  result.raw_set("id", format_id(id))?;
  result.raw_set("width", binding.width)?;
  result.raw_set("height", binding.height)?;
  result.raw_set("hover_move", binding.options.hover_move)?;
  result.raw_set("drag", binding.options.drag)?;
  Ok(result)
}
