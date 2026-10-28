//! Strict Lua bindings for dynamically embedded scrollbox components.

use super::*;
use crate::object_pool::LuaScrollboxBinding;
use tg_service_widget::{ScrollBoxWidget, ScrollBoxWidgetOptions};

const FIELDS: &[&str] = &[
  "width",
  "height",
  "bg",
  "mouse",
  "wheel_vertical",
  "wheel_horizontal",
  "scrollbar_swap_vertical",
  "scrollbar_swap_horizontal",
  "track_shape_vertical",
  "thumb_shape_vertical",
  "track_shape_horizontal",
  "thumb_shape_horizontal",
  "include_scrollbar",
  "always_show_scrollbar",
  "wheel_step_vertical",
  "wheel_step_horizontal",
];

/// Build the read-only scrollbox library.
///
/// # Errors
///
/// Propagate Lua allocation and registration errors.
pub(super) fn scrollbox(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, value) in [
    ("TO_TOP", "to_top"),
    ("TO_BOTTOM", "to_bottom"),
    ("TO_LEFT", "to_left"),
    ("TO_RIGHT", "to_right"),
  ] {
    source.raw_set(name, value)?;
  }
  let create_state = state.clone();
  source.raw_set(
    "create",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.scrollbox.create";
      let table = positional_table(lua, method, values, &["width", "height"], &FIELDS[2..])?;
      let (options, background) = options(
        &table,
        method,
        ScrollBoxWidgetOptions::default(),
        "none".into(),
      )?;
      with_pool_mut(&create_state, method, |pool| {
        if pool.scrollboxes.len() >= 1024 {
          return Err(args::message(method, "scrollbox limit of 1024 was reached"));
        }
        let widget = ScrollBoxWidget::new(&mut pool.ui, options)
          .ok_or_else(|| args::message(method, "invalid scrollbox options"))?;
        pool.next_scrollbox_id += 1;
        let id = pool.next_scrollbox_id;
        let surface = pool.ui.allocate_embedded_slice();
        pool.scrollboxes.insert(
          id,
          LuaScrollboxBinding {
            widget,
            surface,
            background,
          },
        );
        Ok(format_id(id))
      })
    })?,
  )?;
  let draw_state = state.clone();
  source.raw_set(
    "draw",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.scrollbox.draw";
      let table = positional_table(lua, method, values, &["id", "x", "y"], &["slice_layer"])?;
      let id = table_id(&table, method)?;
      let target = parse_draw_target(&table, method, &draw_state)?;
      let x = coordinate(&table, method, "x")?;
      let y = coordinate(&table, method, "y")?;
      let (surface, sequence) = with_pool_mut(&draw_state, method, |pool| {
        let surface = pool
          .scrollboxes
          .get(&id)
          .ok_or_else(|| args::message(method, "unknown scrollbox id"))?
          .surface;
        let sequence = pool
          .place_surface(surface, target, (x, y))
          .map_err(|error| args::message(method, error))?;
        Ok((surface, sequence))
      })?;
      draw::enqueue_draw(
        &draw_state,
        method,
        LuaDrawCommand::Surface {
          id: surface,
          target,
          x,
          y,
          sequence,
        },
      )
    })?,
  )?;
  let set_state = state.clone();
  source.raw_set(
    "set",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.scrollbox.set";
      let mut fields = FIELDS.to_vec();
      fields.extend(["scroll_offset_vertical", "scroll_offset_horizontal"]);
      let table = positional_table(lua, method, values, &["id"], &fields)?;
      let id = table_id(&table, method)?;
      with_pool_mut(&set_state, method, |pool| {
        let Some(binding) = pool.scrollboxes.get_mut(&id) else {
          return Ok(false);
        };
        let (options, background) = options(
          &table,
          method,
          binding.widget.options().clone(),
          binding.background.clone(),
        )?;
        let vertical_value = table.raw_get::<Value>("scroll_offset_vertical")?;
        let horizontal_value = table.raw_get::<Value>("scroll_offset_horizontal")?;
        let has_vertical = !matches!(vertical_value, Value::Nil);
        let has_horizontal = !matches!(horizontal_value, Value::Nil);
        let vertical = offset(vertical_value, method, "scroll_offset_vertical", true, 0)?;
        let horizontal = offset(
          horizontal_value,
          method,
          "scroll_offset_horizontal",
          false,
          0,
        )?;
        binding.widget.set_options(options);
        binding.background = background;
        if has_vertical || has_horizontal {
          binding.widget.scroll_to_axes(
            has_horizontal.then_some(horizontal),
            has_vertical.then_some(vertical),
          );
        }
        Ok(true)
      })
    })?,
  )?;
  let scroll_state = state.clone();
  source.raw_set(
    "scroll",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.scrollbox.scroll";
      let table = positional_table(
        lua,
        method,
        values,
        &["id", "scroll_times_vertical", "scroll_times_horizontal"],
        &[],
      )?;
      let id = table_id(&table, method)?;
      let vertical = table.raw_get::<Value>("scroll_times_vertical")?;
      let horizontal = table.raw_get::<Value>("scroll_times_horizontal")?;
      with_pool_mut(&scroll_state, method, |pool| {
        let Some(binding) = pool.scrollboxes.get_mut(&id) else {
          return Ok(false);
        };
        let mut delta = [0i64; 2];
        let mut endpoint = [None; 2];
        for (index, value, name, step) in [
          (
            0,
            horizontal,
            "scroll_times_horizontal",
            binding.widget.options().wheel_step_horizontal,
          ),
          (
            1,
            vertical,
            "scroll_times_vertical",
            binding.widget.options().wheel_step_vertical,
          ),
        ] {
          if matches!(value, Value::String(_)) {
            endpoint[index] = Some(offset(value, method, name, index == 1, 0)?);
          } else {
            let count = args::integer(value, method, name)?;
            i32::try_from(count)
              .map_err(|_| args::message(method, format!("{name} is out of i32 range")))?;
            delta[index] = count * i64::from(step);
          }
        }
        binding.widget.scroll_by(delta[0], delta[1]);
        if endpoint.iter().any(Option::is_some) {
          binding.widget.scroll_to_axes(endpoint[0], endpoint[1]);
        }
        Ok(true)
      })
    })?,
  )?;
  for name in ["delete", "exists", "get_info"] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let method = format!("widget.scrollbox.{name}");
        let table = positional_table(lua, &method, values, &["id"], &[])?;
        let id = table_id(&table, &method)?;
        with_pool_mut(&state, &method, |pool| match name {
          "delete" => Ok(Value::Boolean(pool.delete_scrollbox(id))),
          "exists" => Ok(Value::Boolean(pool.scrollboxes.contains_key(&id))),
          _ => pool
            .scrollboxes
            .get(&id)
            .map(|binding| info(lua, id, binding).map(Value::Table))
            .unwrap_or(Ok(Value::Nil)),
        })
      })?,
    )?;
  }
  for name in ["clear", "count", "list"] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let method = format!("widget.scrollbox.{name}");
        args::no_args(&method, values)?;
        with_pool_mut(&state, &method, |pool| match name {
          "clear" => {
            pool.clear_scrollboxes();
            Ok(Value::Boolean(true))
          }
          "count" => Ok(Value::Integer(pool.scrollboxes.len() as i64)),
          _ => {
            let result = lua.create_table()?;
            for (index, (id, binding)) in pool.scrollboxes.iter().enumerate() {
              result.raw_set(index + 1, info(lua, *id, binding)?)?;
            }
            result.raw_set("n", pool.scrollboxes.len())?;
            Ok(Value::Table(result))
          }
        })
      })?,
    )?;
  }
  readonly::proxy(lua, source)
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
  args::string(args::required(table, method, "id")?, method, "id")?
    .strip_prefix("scrollbox_")
    .and_then(|value| value.parse().ok())
    .filter(|id| *id > 0)
    .ok_or_else(|| args::message(method, "invalid scrollbox id"))
}
fn format_id(id: u64) -> String {
  format!("scrollbox_{id:03}")
}
fn offset(
  value: Value,
  method: &str,
  name: &str,
  vertical: bool,
  default: i64,
) -> mlua::Result<i64> {
  match value {
    Value::Nil => Ok(default),
    Value::String(value) => match (vertical, value.to_str()?.as_ref()) {
      (true, "to_top") | (false, "to_left") => Ok(0),
      (true, "to_bottom") | (false, "to_right") => Ok(i64::from(u16::MAX)),
      _ => Err(args::message(method, format!("invalid {name} endpoint"))),
    },
    value => {
      let value = args::integer(value, method, name)?;
      i32::try_from(value)
        .map_err(|_| args::message(method, format!("{name} is out of i32 range")))?;
      Ok(value)
    }
  }
}
fn options(
  table: &Table,
  method: &str,
  mut options: ScrollBoxWidgetOptions,
  mut background: String,
) -> mlua::Result<(ScrollBoxWidgetOptions, String)> {
  for (name, stored) in [
    ("width", &mut options.width),
    ("height", &mut options.height),
    ("wheel_step_vertical", &mut options.wheel_step_vertical),
    ("wheel_step_horizontal", &mut options.wheel_step_horizontal),
  ] {
    let value = table.raw_get::<Value>(name)?;
    if !matches!(value, Value::Nil) {
      *stored = positive_u16(table, method, name)?;
    }
  }
  for (name, stored) in [
    ("mouse", &mut options.mouse),
    ("wheel_vertical", &mut options.wheel_vertical),
    ("wheel_horizontal", &mut options.wheel_horizontal),
    (
      "scrollbar_swap_vertical",
      &mut options.scrollbar_swap_vertical,
    ),
    (
      "scrollbar_swap_horizontal",
      &mut options.scrollbar_swap_horizontal,
    ),
    ("include_scrollbar", &mut options.include_scrollbar),
    ("always_show_scrollbar", &mut options.always_show_scrollbar),
  ] {
    *stored = args::optional_bool(table, method, name, *stored)?;
  }
  for (name, stored) in [
    ("track_shape_vertical", &mut options.style.track_char),
    ("thumb_shape_vertical", &mut options.style.thumb_char),
    ("track_shape_horizontal", &mut options.style.h_track_char),
    ("thumb_shape_horizontal", &mut options.style.h_thumb_char),
  ] {
    let value = table.raw_get::<Value>(name)?;
    if !matches!(value, Value::Nil) {
      let text = args::string(value, method, name)?;
      let mut chars = text.chars();
      let character = chars
        .next()
        .filter(|_| chars.next().is_none())
        .ok_or_else(|| args::message(method, format!("{name} must be one character")))?;
      if tg_core_unicode::char_width(character) != 1 {
        return Err(args::message(
          method,
          format!("{name} must occupy one cell"),
        ));
      }
      *stored = character;
    }
  }
  let value = table.raw_get::<Value>("bg")?;
  if !matches!(value, Value::Nil) {
    options.background = parse_color(value.clone(), method, "bg", true)?;
    background = args::string(value, method, "bg")?;
  }
  Ok((options, background))
}
fn info(lua: &Lua, id: u64, binding: &LuaScrollboxBinding) -> mlua::Result<Table> {
  let result = lua.create_table()?;
  let options = binding.widget.options();
  let layout = binding.widget.layout();
  let (x, y) = binding.widget.offset();
  result.raw_set("id", format_id(id))?;
  result.raw_set("bg", binding.background.as_str())?;
  for (name, value) in [
    ("width", options.width),
    ("height", options.height),
    ("content_width", layout.content.width),
    ("content_height", layout.content.height),
    ("scrollbox_width", layout.outer.width),
    ("scrollbox_height", layout.outer.height),
    ("wheel_step_vertical", options.wheel_step_vertical),
    ("wheel_step_horizontal", options.wheel_step_horizontal),
    ("scroll_offset_vertical", y),
    ("scroll_offset_horizontal", x),
  ] {
    result.raw_set(name, value)?;
  }
  for (name, value) in [
    ("mouse", options.mouse),
    ("wheel_vertical", options.wheel_vertical),
    ("wheel_horizontal", options.wheel_horizontal),
    ("scrollbar_swap_vertical", options.scrollbar_swap_vertical),
    (
      "scrollbar_swap_horizontal",
      options.scrollbar_swap_horizontal,
    ),
    ("include_scrollbar", options.include_scrollbar),
    ("always_show_scrollbar", options.always_show_scrollbar),
  ] {
    result.raw_set(name, value)?;
  }
  for (name, value) in [
    ("track_shape_vertical", options.style.track_char),
    ("thumb_shape_vertical", options.style.thumb_char),
    ("track_shape_horizontal", options.style.h_track_char),
    ("thumb_shape_horizontal", options.style.h_thumb_char),
  ] {
    result.raw_set(name, value.to_string())?;
  }
  Ok(result)
}
