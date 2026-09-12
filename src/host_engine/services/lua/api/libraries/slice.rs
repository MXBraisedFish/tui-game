use mlua::{Lua, MultiValue, Table, Value};

use super::*;
use crate::host_engine::services::{
  LuaObjectPool, Size, SliceId, SliceLength, SliceOptions, SliceRect, SliceService, TerminalColor,
  TextColor,
};

const MAX_SLICES: usize = 1024;

#[derive(Clone, Copy)]
enum SliceHandle {
  Base,
  Object(SliceId),
}

pub(super) fn slice(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  install_lifecycle(lua, &source, state.clone())?;
  install_mutations(lua, &source, state.clone())?;
  install_queries(lua, &source, state)?;
  readonly::proxy(lua, source)
}

fn install_lifecycle(lua: &Lua, source: &Table, state: SharedApiState) -> mlua::Result<()> {
  let create_state = state.clone();
  source.raw_set(
    "create",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "slice.create";
      let table = args::named(method, values, &["width", "height", "bg", "layer"])?;
      let width = length(args::required(&table, method, "width")?, method, "width")?;
      let height = length(args::required(&table, method, "height")?, method, "height")?;
      let background = parse_color(table.get::<Value>("bg")?, method, "bg", true)?;
      let layer = optional_layer(&table, method)?;
      with_pool_mut(&create_state, method, |objects| {
        let service = SliceService::new();
        if service.ids(objects.ui()).len() >= MAX_SLICES {
          return Err(args::message(method, "slice limit of 1024 was reached"));
        }
        let id = service
          .create(
            objects.ui_mut(),
            SliceOptions {
              rect: SliceRect {
                x: 0,
                y: 0,
                width,
                height,
              },
              layer,
              background,
              ..Default::default()
            },
          )
          .ok_or_else(|| args::message(method, "invalid slice dimensions"))?;
        // Lua 切片是逐帧提交的资源；创建只保留配置，不使其自动出现在画布上。
        service.set_frame_scoped(objects.ui_mut(), id, true);
        Ok(Value::String(lua.create_string(format_id(id))?))
      })
    })?,
  )?;

  let delete_state = state.clone();
  source.raw_set(
    "delete",
    lua.create_function(move |_, values: MultiValue| {
      let method = "slice.delete";
      match id_argument(values, method)? {
        SliceHandle::Base => Ok(false),
        SliceHandle::Object(id) => with_pool_mut(&delete_state, method, |objects| {
          Ok(SliceService::new().remove(objects.ui_mut(), id))
        }),
      }
    })?,
  )?;

  source.raw_set(
    "clear",
    lua.create_function(move |_, values: MultiValue| {
      let method = "slice.clear";
      args::no_args(method, values)?;
      with_pool_mut(&state, method, |objects| {
        let service = SliceService::new();
        for id in service.ids(objects.ui()) {
          service.remove(objects.ui_mut(), id);
        }
        Ok(true)
      })
    })?,
  )
}

fn install_mutations(lua: &Lua, source: &Table, state: SharedApiState) -> mlua::Result<()> {
  for (name, fields) in [
    ("set", &["id", "width", "height", "bg", "layer"][..]),
    ("set_size", &["id", "width", "height"][..]),
    ("set_width", &["id", "width"][..]),
    ("set_height", &["id", "height"][..]),
    ("set_background", &["id", "bg"][..]),
    ("set_layer", &["id", "layer"][..]),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method: &'static str = match name {
          "set" => "slice.set",
          "set_size" => "slice.set_size",
          "set_width" => "slice.set_width",
          "set_height" => "slice.set_height",
          "set_background" => "slice.set_background",
          _ => "slice.set_layer",
        };
        let table = args::named(method, values, fields)?;
        let handle = table_id(&table, method)?;
        let width = optional_length(&table, method, "width")?;
        let height = optional_length(&table, method, "height")?;
        let background = optional_background(&table, method)?;
        let layer = optional_layer(&table, method)?;
        let SliceHandle::Object(id) = handle else {
          return Ok(false);
        };
        with_pool_mut(&state, method, |objects| {
          let service = SliceService::new();
          let Some(mut rect) = service.configured_rect(objects.ui(), id) else {
            return Ok(false);
          };
          if let Some(width) = width {
            rect.width = width;
          }
          if let Some(height) = height {
            rect.height = height;
          }
          if (width.is_some() || height.is_some()) && !service.set_rect(objects.ui_mut(), id, rect)
          {
            return Ok(false);
          }
          if let Some(background) = background
            && !service.set_background(objects.ui_mut(), id, background)
          {
            return Ok(false);
          }
          if let Some(layer) = layer
            && !service.set_layer(objects.ui_mut(), id, layer)
          {
            return Ok(false);
          }
          Ok(true)
        })
      })?,
    )?;
  }

  source.raw_set(
    "draw",
    lua.create_function(move |_, values: MultiValue| {
      let method = "slice.draw";
      let table = args::named(method, values, &["id", "x", "y"])?;
      let SliceHandle::Object(id) = table_id(&table, method)? else {
        return Err(args::message(method, "base layer cannot be positioned"));
      };
      let x = checked_i32(
        args::integer(args::required(&table, method, "x")?, method, "x")?,
        method,
        "x",
      )?;
      let y = checked_i32(
        args::integer(args::required(&table, method, "y")?, method, "y")?,
        method,
        "y",
      )?;
      with_pool_mut(&state, method, |objects| {
        if SliceService::new().draw(objects.ui_mut(), id, x, y) {
          Ok(())
        } else {
          Err(args::message(method, "unknown slice id"))
        }
      })
    })?,
  )
}

fn install_queries(lua: &Lua, source: &Table, state: SharedApiState) -> mlua::Result<()> {
  let exists_state = state.clone();
  source.raw_set(
    "exists",
    lua.create_function(move |_, values: MultiValue| {
      let method = "slice.exists";
      match id_argument(values, method)? {
        SliceHandle::Base => Ok(true),
        SliceHandle::Object(id) => with_pool(&exists_state, method, |objects| {
          Ok(SliceService::new().exists(objects.ui(), id))
        }),
      }
    })?,
  )?;

  for name in [
    "get_size",
    "get_width",
    "get_height",
    "get_layer",
    "get_background",
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let method: &'static str = match name {
          "get_size" => "slice.get_size",
          "get_width" => "slice.get_width",
          "get_height" => "slice.get_height",
          "get_layer" => "slice.get_layer",
          _ => "slice.get_background",
        };
        let handle = id_argument(values, method)?;
        query_value(lua, &state, method, handle, name)
      })?,
    )?;
  }

  let info_state = state.clone();
  source.raw_set(
    "get_info",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "slice.get_info";
      let handle = id_argument(values, method)?;
      info_value(lua, &info_state, method, handle)
    })?,
  )?;

  let list_state = state.clone();
  source.raw_set(
    "list",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "slice.list";
      args::no_args(method, values)?;
      let size = list_state.borrow().context.base_size;
      with_pool(&list_state, method, |objects| {
        let service = SliceService::new();
        let result = lua.create_table()?;
        let ids = service.ids_by_layer(objects.ui());
        for (index, id) in ids.iter().copied().enumerate() {
          result.raw_set(index + 1, object_info(lua, objects, id, size)?)?;
        }
        result.raw_set("n", ids.len())?;
        Ok(result)
      })
    })?,
  )?;

  source.raw_set(
    "count",
    lua.create_function(move |_, values: MultiValue| {
      let method = "slice.count";
      args::no_args(method, values)?;
      with_pool(&state, method, |objects| {
        Ok(SliceService::new().ids(objects.ui()).len())
      })
    })?,
  )
}

fn query_value(
  lua: &Lua,
  state: &SharedApiState,
  method: &str,
  handle: SliceHandle,
  query: &str,
) -> mlua::Result<Value> {
  let base = state.borrow().context.base_size;
  match handle {
    SliceHandle::Base => match query {
      "get_size" => Ok(Value::Table(size_table(lua, base.width, base.height)?)),
      "get_width" => Ok(Value::Integer(i64::from(base.width))),
      "get_height" => Ok(Value::Integer(i64::from(base.height))),
      "get_layer" => Ok(Value::Integer(0)),
      _ => Ok(Value::String(lua.create_string("transparent")?)),
    },
    SliceHandle::Object(id) => with_pool(state, method, |objects| {
      let service = SliceService::new();
      let Some(rect) = service.configured_rect(objects.ui(), id) else {
        return Ok(Value::Nil);
      };
      match query {
        "get_size" => Ok(Value::Table(size_table(
          lua,
          resolve_length(rect.width, base.width),
          resolve_length(rect.height, base.height),
        )?)),
        "get_width" => Ok(Value::Integer(i64::from(resolve_length(
          rect.width, base.width,
        )))),
        "get_height" => Ok(Value::Integer(i64::from(resolve_length(
          rect.height,
          base.height,
        )))),
        "get_layer" => Ok(
          service
            .layer(objects.ui(), id)
            .map_or(Value::Nil, |value| Value::Integer(i64::from(value))),
        ),
        _ => match service.background(objects.ui(), id) {
          Some(value) => Ok(Value::String(lua.create_string(background_name(value))?)),
          None => Ok(Value::Nil),
        },
      }
    }),
  }
}

fn info_value(
  lua: &Lua,
  state: &SharedApiState,
  method: &str,
  handle: SliceHandle,
) -> mlua::Result<Value> {
  let size = state.borrow().context.base_size;
  match handle {
    SliceHandle::Base => {
      let info = lua.create_table()?;
      info.raw_set("id", "base")?;
      info.raw_set("width", size.width)?;
      info.raw_set("height", size.height)?;
      info.raw_set("layer", 0)?;
      info.raw_set("bg", "transparent")?;
      Ok(Value::Table(info))
    }
    SliceHandle::Object(id) => with_pool(state, method, |objects| {
      if !SliceService::new().exists(objects.ui(), id) {
        return Ok(Value::Nil);
      }
      Ok(Value::Table(object_info(lua, objects, id, size)?))
    }),
  }
}

fn object_info(lua: &Lua, objects: &LuaObjectPool, id: SliceId, base: Size) -> mlua::Result<Table> {
  let service = SliceService::new();
  let rect = service.configured_rect(objects.ui(), id).unwrap();
  let info = lua.create_table()?;
  info.raw_set("id", format_id(id))?;
  info.raw_set("width", resolve_length(rect.width, base.width))?;
  info.raw_set("height", resolve_length(rect.height, base.height))?;
  info.raw_set("layer", service.layer(objects.ui(), id).unwrap_or_default())?;
  info.raw_set(
    "bg",
    background_name(service.background(objects.ui(), id).unwrap_or(None)),
  )?;
  Ok(info)
}

fn size_table(lua: &Lua, width: u16, height: u16) -> mlua::Result<Table> {
  let result = lua.create_table()?;
  result.raw_set("width", width)?;
  result.raw_set("height", height)?;
  Ok(result)
}

fn length(value: Value, method: &str, name: &str) -> mlua::Result<SliceLength> {
  let value = args::integer(value, method, name)?;
  u16::try_from(value)
    .ok()
    .filter(|value| *value > 0)
    .map(SliceLength::Fixed)
    .ok_or_else(|| args::message(method, format!("{name} must be in 1..=65535")))
}

fn optional_length(table: &Table, method: &str, name: &str) -> mlua::Result<Option<SliceLength>> {
  match table.get::<Value>(name)? {
    Value::Nil => Ok(None),
    value => length(value, method, name).map(Some),
  }
}

fn optional_layer(table: &Table, method: &str) -> mlua::Result<Option<i32>> {
  let Some(layer) = args::optional_integer(table, method, "layer", None)? else {
    return Ok(None);
  };
  let layer = checked_i32(layer, method, "layer")?;
  if layer < 1 {
    return Err(args::message(method, "layer must be a positive integer"));
  }
  Ok(Some(layer))
}

fn optional_background(table: &Table, method: &str) -> mlua::Result<Option<Option<TextColor>>> {
  match table.get::<Value>("bg")? {
    Value::Nil => Ok(None),
    value => parse_color(value, method, "bg", true).map(Some),
  }
}

pub(super) fn resolve_length(length: SliceLength, total: u16) -> u16 {
  match length {
    SliceLength::Fixed(value) => value,
    SliceLength::Auto => total,
    SliceLength::Percent(value) => (u32::from(total) * u32::from(value) / 100) as u16,
  }
}

fn id_argument(values: MultiValue, method: &str) -> mlua::Result<SliceHandle> {
  let value = args::string(args::one(method, "id", values)?, method, "id")?;
  parse_handle(&value, method, "id")
}

fn table_id(table: &Table, method: &str) -> mlua::Result<SliceHandle> {
  let value = args::string(args::required(table, method, "id")?, method, "id")?;
  parse_handle(&value, method, "id")
}

fn parse_handle(value: &str, method: &str, name: &str) -> mlua::Result<SliceHandle> {
  if value == "base" {
    return Ok(SliceHandle::Base);
  }
  parse_id(value, method, name).map(SliceHandle::Object)
}

pub(super) fn parse_id(value: &str, method: &str, name: &str) -> mlua::Result<SliceId> {
  let raw = value
    .strip_prefix("slice_")
    .and_then(|value| value.parse::<u64>().ok())
    .filter(|value| *value > 0)
    .ok_or_else(|| args::message(method, format!("invalid {name} slice id")))?;
  Ok(SliceId(raw))
}

fn format_id(id: SliceId) -> String {
  format!("slice_{:03}", id.0)
}

fn checked_i32(value: i64, method: &str, name: &str) -> mlua::Result<i32> {
  i32::try_from(value).map_err(|_| args::message(method, format!("{name} is out of i32 range")))
}

fn background_name(color: Option<TextColor>) -> String {
  match color {
    None => "none".to_string(),
    Some(TextColor::Transparent) => "transparent".to_string(),
    Some(TextColor::Rgb { r, g, b } | TextColor::ForceRgb { r, g, b }) => {
      format!("rgb({r},{g},{b})")
    }
    Some(TextColor::Terminal(color)) => match color {
      TerminalColor::Black => "black",
      TerminalColor::Red => "red",
      TerminalColor::Green => "green",
      TerminalColor::Yellow => "yellow",
      TerminalColor::Blue => "blue",
      TerminalColor::Magenta => "magenta",
      TerminalColor::Cyan => "cyan",
      TerminalColor::BrightBlack => "gray",
      TerminalColor::White => "bright_gray",
      TerminalColor::BrightRed => "bright_red",
      TerminalColor::BrightGreen => "bright_green",
      TerminalColor::BrightYellow => "bright_yellow",
      TerminalColor::BrightBlue => "bright_blue",
      TerminalColor::BrightMagenta => "bright_magenta",
      TerminalColor::BrightCyan => "bright_cyan",
      TerminalColor::BrightWhite => "white",
    }
    .to_string(),
  }
}

fn with_pool<R>(
  state: &SharedApiState,
  method: &str,
  operation: impl FnOnce(&LuaObjectPool) -> mlua::Result<R>,
) -> mlua::Result<R> {
  let objects = state
    .borrow()
    .objects
    .upgrade()
    .ok_or_else(|| args::message(method, "session object pool is unavailable"))?;
  let objects = objects
    .try_borrow()
    .map_err(|_| args::message(method, "session object pool is busy"))?;
  operation(
    objects
      .as_ref()
      .ok_or_else(|| args::message(method, "session object pool is unavailable"))?,
  )
}

fn with_pool_mut<R>(
  state: &SharedApiState,
  method: &str,
  operation: impl FnOnce(&mut LuaObjectPool) -> mlua::Result<R>,
) -> mlua::Result<R> {
  let objects = state
    .borrow()
    .objects
    .upgrade()
    .ok_or_else(|| args::message(method, "session object pool is unavailable"))?;
  let mut objects = objects
    .try_borrow_mut()
    .map_err(|_| args::message(method, "session object pool is busy"))?;
  operation(
    objects
      .as_mut()
      .ok_or_else(|| args::message(method, "session object pool is unavailable"))?,
  )
}
