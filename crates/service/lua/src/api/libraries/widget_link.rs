//! Strict Lua bindings for clickable file, directory, and web links.

use super::*;
use crate::object_pool::LuaLinkBinding;
use tg_core_style::TextStyle;
use tg_service_widget::{HyperlinkId, HyperlinkService, is_valid_link_target};

const MAX_LINKS: usize = 1024;
const FIELDS: &[&str] = &[
  "text",
  "link",
  "fg",
  "bg",
  "hover_fg",
  "hover_bg",
  "mouse",
  "copy",
  "bold",
  "italic",
  "underline",
  "strike",
  "blink",
  "reverse",
  "hidden",
  "dim",
];

/// Build the read-only link library.
///
/// # Errors
///
/// Propagate Lua allocation and function registration errors.
pub(super) fn link(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  let create_state = state.clone();
  source.raw_set(
    "create",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.link.create";
      let table = positional_table(lua, method, values, &["text", "link"], &FIELDS[2..])?;
      let binding = options(
        &table,
        method,
        LuaLinkBinding::new(
          args::string(args::required(&table, method, "text")?, method, "text")?,
          args::string(args::required(&table, method, "link")?, method, "link")?,
        ),
      )?;
      with_pool_mut(&create_state, method, |pool| {
        if pool.links.len() >= MAX_LINKS {
          return Err(args::message(method, "link limit of 1024 was reached"));
        }
        let id = HyperlinkService::new()
          .create(&mut pool.ui, binding.options.clone())
          .ok_or_else(|| args::message(method, "link and text cannot be empty"))?;
        pool.links.insert(id.0, binding);
        Ok(format_id(id.0))
      })
    })?,
  )?;
  let set_state = state.clone();
  source.raw_set(
    "set",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.link.set";
      let table = positional_table(lua, method, values, &["id"], FIELDS)?;
      let id = table_id(&table, method)?;
      with_pool_mut(&set_state, method, |pool| {
        let Some(binding) = pool.links.get(&id).cloned() else {
          return Ok(false);
        };
        let binding = options(&table, method, binding)?;
        HyperlinkService::new().set_options(&mut pool.ui, HyperlinkId(id), binding.options.clone());
        pool.links.insert(id, binding);
        Ok(true)
      })
    })?,
  )?;
  let draw_state = state.clone();
  source.raw_set(
    "draw",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.link.draw";
      let table = positional_table(lua, method, values, &["id", "x", "y"], &["slice_layer"])?;
      let id = table_id(&table, method)?;
      let coordinate = |name| {
        let value = args::integer(args::required(&table, method, name)?, method, name)?;
        i32::try_from(value)
          .map_err(|_| args::message(method, format!("{name} is out of i32 range")))
      };
      let command = LuaDrawCommand::Link {
        id,
        target: parse_draw_target(&table, method, &draw_state)?,
        x: coordinate("x")?,
        y: coordinate("y")?,
      };
      with_pool(&draw_state, method, |pool| {
        if pool.links.contains_key(&id) {
          Ok(())
        } else {
          Err(args::message(method, "unknown link id"))
        }
      })?;
      draw::enqueue_draw(&draw_state, method, command)
    })?,
  )?;
  for (name, method) in [
    ("delete", "widget.link.delete"),
    ("exists", "widget.link.exists"),
    ("get_info", "widget.link.get_info"),
    ("click", "widget.link.click"),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let table = positional_table(lua, method, values, &["id"], &[])?;
        let id = table_id(&table, method)?;
        if name == "click" && state.borrow().context.session_kind != LuaSessionKind::Game {
          return Ok(Value::Boolean(false));
        }

        with_pool_mut(&state, method, |pool| match name {
          "delete" => {
            let removed = pool.links.remove(&id).is_some();
            if removed {
              HyperlinkService::new().remove(&mut pool.ui, HyperlinkId(id));
            }
            Ok(Value::Boolean(removed))
          }
          "exists" => Ok(Value::Boolean(pool.links.contains_key(&id))),
          "click" => Ok(Value::Boolean(
            pool.links.contains_key(&id)
              && HyperlinkService::new().click(&mut pool.ui, HyperlinkId(id)),
          )),
          _ => pool
            .links
            .get(&id)
            .map(|binding| info(lua, id, binding).map(Value::Table))
            .unwrap_or(Ok(Value::Nil)),
        })
      })?,
    )?;
  }
  for (name, method) in [
    ("clear", "widget.link.clear"),
    ("count", "widget.link.count"),
    ("list", "widget.link.list"),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        args::no_args(method, values)?;
        with_pool_mut(&state, method, |pool| match name {
          "clear" => {
            for id in std::mem::take(&mut pool.links).into_keys() {
              HyperlinkService::new().remove(&mut pool.ui, HyperlinkId(id));
            }
            Ok(Value::Boolean(true))
          }
          "count" => Ok(Value::Integer(pool.links.len() as i64)),
          _ => {
            let table = lua.create_table()?;
            for (index, (id, binding)) in pool.links.iter().enumerate() {
              table.raw_set(index + 1, info(lua, *id, binding)?)?;
            }
            table.raw_set("n", pool.links.len())?;
            Ok(Value::Table(table))
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
  mut binding: LuaLinkBinding,
) -> mlua::Result<LuaLinkBinding> {
  binding.options.text =
    args::optional_string(table, method, "text", Some(&binding.options.text))?.unwrap();
  binding.options.link =
    args::optional_string(table, method, "link", Some(&binding.options.link))?.unwrap();
  if binding.options.text.is_empty() {
    return Err(args::message(method, "text cannot be empty"));
  }
  if !is_valid_link_target(&binding.options.link) {
    return Err(args::message(
      method,
      "link must be an asset-relative path or an http/https URL",
    ));
  }
  let mut hover = binding
    .options
    .hover_style
    .clone()
    .unwrap_or_else(|| binding.options.style.clone());
  for (name, stored, parsed, background) in [
    (
      "fg",
      &mut binding.fg,
      &mut binding.options.style.foreground,
      false,
    ),
    (
      "bg",
      &mut binding.bg,
      &mut binding.options.style.background,
      true,
    ),
    (
      "hover_fg",
      &mut binding.hover_fg,
      &mut hover.foreground,
      false,
    ),
    (
      "hover_bg",
      &mut binding.hover_bg,
      &mut hover.background,
      true,
    ),
  ] {
    let value = table.raw_get::<Value>(name)?;
    if !matches!(value, Value::Nil) {
      *parsed = parse_color(value.clone(), method, name, background)?;
      *stored = args::string(value, method, name)?;
    }
  }
  binding.copy = args::optional_bool(table, method, "copy", binding.copy)?;
  binding.options.mouse = args::optional_bool(table, method, "mouse", binding.options.mouse)?;
  let style = &mut binding.options.style;
  style.bold = args::optional_bool(table, method, "bold", style.bold)?;
  style.italic = args::optional_bool(table, method, "italic", style.italic)?;
  style.underline = args::optional_bool(table, method, "underline", style.underline)?;
  style.strike = args::optional_bool(table, method, "strike", style.strike)?;
  style.blink = args::optional_bool(table, method, "blink", style.blink)?;
  style.reverse = args::optional_bool(table, method, "reverse", style.reverse)?;
  style.hidden = args::optional_bool(table, method, "hidden", style.hidden)?;
  style.dim = args::optional_bool(table, method, "dim", style.dim)?;
  binding.options.hover_style = Some(TextStyle {
    foreground: hover.foreground,
    background: hover.background,
    ..style.clone()
  });
  Ok(binding)
}
fn table_id(table: &Table, method: &str) -> mlua::Result<u64> {
  args::string(args::required(table, method, "id")?, method, "id")?
    .strip_prefix("link_")
    .and_then(|value| value.parse().ok())
    .filter(|id| *id > 0)
    .ok_or_else(|| args::message(method, "invalid link id"))
}
fn format_id(id: u64) -> String {
  format!("link_{id:03}")
}
fn info(lua: &Lua, id: u64, binding: &LuaLinkBinding) -> mlua::Result<Table> {
  let result = lua.create_table()?;
  result.raw_set("id", format_id(id))?;
  for (name, value) in [
    ("text", binding.options.text.as_str()),
    ("link", binding.options.link.as_str()),
    ("fg", binding.fg.as_str()),
    ("bg", binding.bg.as_str()),
    ("hover_fg", binding.hover_fg.as_str()),
    ("hover_bg", binding.hover_bg.as_str()),
  ] {
    result.raw_set(name, value)?;
  }
  let style = &binding.options.style;
  for (name, value) in [
    ("mouse", binding.options.mouse),
    ("copy", binding.copy),
    ("bold", style.bold),
    ("italic", style.italic),
    ("underline", style.underline),
    ("strike", style.strike),
    ("blink", style.blink),
    ("reverse", style.reverse),
    ("hidden", style.hidden),
    ("dim", style.dim),
  ] {
    result.raw_set(name, value)?;
  }
  Ok(result)
}
