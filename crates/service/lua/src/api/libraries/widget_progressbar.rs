//! Strict Lua bindings for single-row progress indicators.
use super::*;
use crate::object_pool::LuaProgressbarBinding;
use tg_service_widget::{ProgressBarId, ProgressBarService};
const FIELDS: &[&str] = &[
  "width",
  "completed_ratio",
  "completed_shape",
  "completed_fg",
  "preview_ratio",
  "preview_shape",
  "preview_fg",
  "remaining_shape",
  "remaining_fg",
];

/// Build the read-only progress bar library.
///
/// # Errors
///
/// Propagate Lua allocation and function registration errors.
pub(super) fn progressbar(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  let create_state = state.clone();
  source.raw_set(
    "create",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.progressbar.create";
      let table = positional_table(lua, method, values, &["width"], &FIELDS[1..])?;
      let binding = options(&table, method, LuaProgressbarBinding::default())?;
      with_pool_mut(&create_state, method, |pool| {
        if pool.progressbars.len() >= 1024 {
          return Err(args::message(
            method,
            "progressbar limit of 1024 was reached",
          ));
        }
        let service = ProgressBarService::new();
        let id = service
          .create(&mut pool.ui, binding.options.clone())
          .ok_or_else(|| args::message(method, "invalid progress bar shapes"))?;
        service.set_progress(
          &mut pool.ui,
          id,
          binding.completed_ratio as f32,
          binding.preview_ratio as f32,
        );
        pool.progressbars.insert(id.0, binding);
        Ok(format_id(id.0))
      })
    })?,
  )?;
  let set_state = state.clone();
  source.raw_set(
    "set",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.progressbar.set";
      let table = positional_table(lua, method, values, &["id"], FIELDS)?;
      let id = table_id(&table, method)?;
      with_pool_mut(&set_state, method, |pool| {
        let Some(binding) = pool.progressbars.get(&id).cloned() else {
          return Ok(false);
        };
        let binding = options(&table, method, binding)?;
        let service = ProgressBarService::new();
        service.set_options(&mut pool.ui, ProgressBarId(id), binding.options.clone());
        service.set_progress(
          &mut pool.ui,
          ProgressBarId(id),
          binding.completed_ratio as f32,
          binding.preview_ratio as f32,
        );
        pool.progressbars.insert(id, binding);
        Ok(true)
      })
    })?,
  )?;
  let draw_state = state.clone();
  source.raw_set(
    "draw",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.progressbar.draw";
      let table = positional_table(lua, method, values, &["id", "x", "y"], &["slice_layer"])?;
      let id = table_id(&table, method)?;
      let coordinate = |name| {
        i32::try_from(args::integer(
          args::required(&table, method, name)?,
          method,
          name,
        )?)
        .map_err(|_| args::message(method, format!("{name} is out of i32 range")))
      };
      let command = LuaDrawCommand::Progressbar {
        id,
        target: parse_draw_target(&table, method, &draw_state)?,
        x: coordinate("x")?,
        y: coordinate("y")?,
      };
      with_pool(&draw_state, method, |pool| {
        if pool.progressbars.contains_key(&id) {
          Ok(())
        } else {
          Err(args::message(method, "unknown progressbar id"))
        }
      })?;
      draw::enqueue_draw(&draw_state, method, command)
    })?,
  )?;
  for name in ["delete", "get_info", "exists", "clear", "count", "list"] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let method = format!("widget.progressbar.{name}");
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
          "delete" => {
            let id = id.unwrap();
            let removed = pool.progressbars.remove(&id).is_some();
            if removed {
              ProgressBarService::new().remove(&mut pool.ui, ProgressBarId(id));
            }
            Ok(Value::Boolean(removed))
          }
          "exists" => Ok(Value::Boolean(pool.progressbars.contains_key(&id.unwrap()))),
          "get_info" => pool
            .progressbars
            .get(&id.unwrap())
            .map(|binding| info(lua, id.unwrap(), binding).map(Value::Table))
            .unwrap_or(Ok(Value::Nil)),
          "clear" => {
            for id in std::mem::take(&mut pool.progressbars).into_keys() {
              ProgressBarService::new().remove(&mut pool.ui, ProgressBarId(id));
            }
            Ok(Value::Boolean(true))
          }
          "count" => Ok(Value::Integer(pool.progressbars.len() as i64)),
          _ => {
            let result = lua.create_table()?;
            for (index, (id, binding)) in pool.progressbars.iter().enumerate() {
              result.raw_set(index + 1, info(lua, *id, binding)?)?;
            }
            result.raw_set("n", pool.progressbars.len())?;
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
  mut binding: LuaProgressbarBinding,
) -> mlua::Result<LuaProgressbarBinding> {
  if !matches!(table.raw_get::<Value>("width")?, Value::Nil) {
    binding.width = positive_u16(table, method, "width")?;
  }
  for (name, stored) in [
    ("completed_ratio", &mut binding.completed_ratio),
    ("preview_ratio", &mut binding.preview_ratio),
  ] {
    let value = table.raw_get::<Value>(name)?;
    if !matches!(value, Value::Nil) {
      let ratio = args::number(value, method, name)?;
      if !ratio.is_finite() || !(0.0..=1.0).contains(&ratio) {
        return Err(args::message(
          method,
          format!("{name} must be a finite number in 0..=1"),
        ));
      }
      *stored = ratio;
    }
  }
  for (name, stored) in [
    ("completed_shape", &mut binding.options.completed.ch),
    ("preview_shape", &mut binding.options.preview.ch),
    ("remaining_shape", &mut binding.options.remaining.ch),
  ] {
    let value = table.raw_get::<Value>(name)?;
    if !matches!(value, Value::Nil) {
      let text = args::string(value, method, name)?;
      let mut characters = text.chars();
      let character = characters
        .next()
        .filter(|_| characters.next().is_none())
        .filter(|character| tg_core_unicode::char_width(*character) == 1)
        .ok_or_else(|| {
          args::message(
            method,
            format!("{name} must be one character occupying one cell"),
          )
        })?;
      *stored = character;
    }
  }
  for (name, stored, color) in [
    (
      "completed_fg",
      &mut binding.completed_fg,
      &mut binding.options.completed.style.foreground,
    ),
    (
      "preview_fg",
      &mut binding.preview_fg,
      &mut binding.options.preview.style.foreground,
    ),
    (
      "remaining_fg",
      &mut binding.remaining_fg,
      &mut binding.options.remaining.style.foreground,
    ),
  ] {
    let value = table.raw_get::<Value>(name)?;
    if !matches!(value, Value::Nil) {
      *color = parse_color(value.clone(), method, name, false)?;
      *stored = args::string(value, method, name)?;
    }
  }
  Ok(binding)
}
fn table_id(table: &Table, method: &str) -> mlua::Result<u64> {
  args::string(args::required(table, method, "id")?, method, "id")?
    .strip_prefix("progressbar_")
    .and_then(|id| id.parse().ok())
    .filter(|id| *id > 0)
    .ok_or_else(|| args::message(method, "invalid progressbar id"))
}
fn format_id(id: u64) -> String {
  format!("progressbar_{id:03}")
}
fn info(lua: &Lua, id: u64, binding: &LuaProgressbarBinding) -> mlua::Result<Table> {
  let result = lua.create_table()?;
  result.raw_set("id", format_id(id))?;
  result.raw_set("width", binding.width)?;
  result.raw_set("completed_ratio", binding.completed_ratio)?;
  result.raw_set("preview_ratio", binding.preview_ratio)?;
  for (name, value) in [
    ("completed_shape", binding.options.completed.ch),
    ("preview_shape", binding.options.preview.ch),
    ("remaining_shape", binding.options.remaining.ch),
  ] {
    result.raw_set(name, value.to_string())?;
  }
  for (name, value) in [
    ("completed_fg", &binding.completed_fg),
    ("preview_fg", &binding.preview_fg),
    ("remaining_fg", &binding.remaining_fg),
  ] {
    result.raw_set(name, value.as_str())?;
  }
  Ok(result)
}
