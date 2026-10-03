//! Lua measurement library bindings with validated arguments and session-owned host access.

use super::*;

/// Build and register the Lua measurement API in the supplied VM and host context.
///
/// # Errors
///
/// Propagate Lua allocation, table construction, or function registration errors while installing
/// this library.
pub(super) fn measurement(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, result) in [
    ("get_text_size", 0_u8),
    ("get_text_width", 1_u8),
    ("get_text_height", 2_u8),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let method = match result {
          0 => "measurement.get_text_size",
          1 => "measurement.get_text_width",
          _ => "measurement.get_text_height",
        };
        let table = measurement_text_parameters(lua, method, values)?;
        let params = parse_draw_text_params(&table, method, &state.borrow().context, false)?;
        let (width, height) = tg_service_text_layout::measure_draw_text(&params);
        match result {
          0 => Ok(MultiValue::from_vec(vec![
            Value::Integer(width as i64),
            Value::Integer(height as i64),
          ])),
          1 => Ok(MultiValue::from_vec(vec![Value::Integer(width as i64)])),
          _ => Ok(MultiValue::from_vec(vec![Value::Integer(height as i64)])),
        }
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}

/// Build validated draw parameters from the trailing Lua option table.
///
/// # Arguments
///
/// * `lua` - The Lua VM in which values and callbacks are created.
/// * `method` - The script-visible method name included in argument errors.
/// * `values` - The input values in their supplied order.
///
/// # Errors
///
/// Return a Lua argument error for unknown option fields, invalid positions, styles, wrapping, or
/// drawing destinations.
pub(super) fn draw_text_parameters(
  lua: &Lua,
  method: &str,
  values: MultiValue,
) -> mlua::Result<Table> {
  positional_table(
    lua,
    method,
    values,
    &["x", "y", "text"],
    &[
      "fg",
      "bg",
      "horizontal_align",
      "auto_wrap",
      "word_wrap",
      "max_height",
      "max_width",
      "overflow_marker",
      "rich_params",
      "bold",
      "italic",
      "underline",
      "strike",
      "blink",
      "reverse",
      "hidden",
      "dim",
      "text_mode",
      "slice_layer",
    ],
  )
}

/// Convert required Lua positions and strict options into the internal parameter table.
///
/// # Arguments
///
/// * `lua` - The Lua VM in which values and callbacks are created.
/// * `method` - The script-visible method name included in argument errors.
/// * `values` - The input values in their supplied order.
/// * `required_names` - Required positional parameter names in their call order.
/// * `option_fields` - The complete list of accepted trailing option keys.
///
/// # Errors
///
/// Return a Lua argument error for missing positions or an invalid strict trailing options table.
pub(super) fn positional_table(
  lua: &Lua,
  method: &str,
  values: MultiValue,
  required_names: &[&str],
  option_fields: &[&str],
) -> mlua::Result<Table> {
  let parsed = args::positional(lua, method, values, required_names, option_fields)?;
  let table = lua.create_table()?;
  for (index, name) in required_names.iter().enumerate() {
    table.raw_set(*name, parsed.required(index, method, name)?)?;
  }
  for pair in parsed.options().clone().pairs::<String, Value>() {
    let (name, value) = pair?;
    table.raw_set(name, value)?;
  }
  Ok(table)
}

fn measurement_text_parameters(lua: &Lua, method: &str, values: MultiValue) -> mlua::Result<Table> {
  positional_table(
    lua,
    method,
    values,
    &["text"],
    &[
      "horizontal_align",
      "auto_wrap",
      "word_wrap",
      "max_height",
      "max_width",
      "overflow_marker",
      "rich_params",
      "text_mode",
    ],
  )
}

/// Validate draw positions, alignment, wrapping, styles, and the destination surface.
///
/// # Arguments
///
/// * `table` - The Lua table to inspect or convert.
/// * `method` - The script-visible method name included in argument errors.
/// * `context` - The state and services needed for the operation.
/// * `include_position` - The include position.
///
/// # Errors
///
/// Return a Lua argument error when a draw parameter has an unsupported type, value, or
/// destination.
pub(super) fn parse_draw_text_params(
  table: &Table,
  method: &str,
  context: &super::LuaApiContext,
  include_position: bool,
) -> mlua::Result<DrawTextParams> {
  let text = args::dynamic_text(args::required(table, method, "text")?, method, "text")?;
  let mode = args::optional_string(table, method, "text_mode", Some("auto"))?.unwrap();
  let text_mode = match mode.as_str() {
    "auto" => TextMode::Auto,
    "plain_text" => TextMode::Plain,
    "rich_text" => TextMode::Rich,
    _ => return Err(args::message(method, "invalid text_mode constant")),
  };
  let horizontal = args::optional_string(table, method, "horizontal_align", Some("left"))?.unwrap();
  let line_align = match horizontal.as_str() {
    "left" | "auto" => TextAlign::Left,
    "horizontal_center" | "center" => TextAlign::Center,
    "right" => TextAlign::Right,
    _ => return Err(args::message(method, "invalid horizontal_align constant")),
  };
  let wrap_mode = match table.get::<Value>("auto_wrap")? {
    Value::Nil => TextWrapMode::Auto,
    Value::Boolean(true) => TextWrapMode::Auto,
    Value::Boolean(false) => TextWrapMode::Normal,
    value => return Err(args::invalid(method, "auto_wrap", "boolean or nil", &value)),
  };
  let max_width = optional_positive_u16(table, method, "max_width")?;
  let max_height = optional_positive_u16(table, method, "max_height")?;
  let mut rich_params = rich_text_params(table.get::<Value>("rich_params")?, method)?;
  let parses_rich_text = match text_mode {
    TextMode::Rich => true,
    TextMode::Auto => text.starts_with("f%"),
    TextMode::Plain => false,
  };
  let needs_user_keys = parses_rich_text && text.contains("{key:");
  let needs_default_keys = parses_rich_text && text.contains("{key_default:");
  if needs_user_keys || needs_default_keys {
    let params = rich_params.get_or_insert_with(Default::default);
    if needs_user_keys {
      params.key_actions.clone_from(&context.key_actions);
    }
    if needs_default_keys {
      params
        .key_default_actions
        .clone_from(&context.key_default_actions);
    }
  }
  let (x, y) = if include_position {
    (
      args::integer(args::required(table, method, "x")?, method, "x")?,
      args::integer(args::required(table, method, "y")?, method, "y")?,
    )
  } else {
    (0, 0)
  };
  Ok(DrawTextParams {
    x: x.clamp(0, u16::MAX as i64) as u16,
    y: y.clamp(0, u16::MAX as i64) as u16,
    text,
    text_mode,
    params: rich_params,
    fg: parse_color(table.get::<Value>("fg")?, method, "fg", false)?,
    bg: parse_color(table.get::<Value>("bg")?, method, "bg", true)?,
    line_align,
    wrap_mode,
    non_truncate_word_wrap: args::optional_bool(table, method, "word_wrap", true)?,
    max_width,
    max_height,
    overflow_marker: args::optional_dynamic_text(table, method, "overflow_marker", Some("..."))?,
    bold: args::optional_bool(table, method, "bold", false)?,
    italic: args::optional_bool(table, method, "italic", false)?,
    underline: args::optional_bool(table, method, "underline", false)?,
    strike: args::optional_bool(table, method, "strike", false)?,
    blink: args::optional_bool(table, method, "blink", false)?,
    reverse: args::optional_bool(table, method, "reverse", false)?,
    hidden: args::optional_bool(table, method, "hidden", false)?,
    dim: args::optional_bool(table, method, "dim", false)?,
  })
}

/// Validate and convert a Lua color argument into a terminal color.
///
/// # Arguments
///
/// * `value` - The value to store or convert.
/// * `method` - The script-visible method name included in argument errors.
/// * `name` - The name used to identify the object or field.
/// * `background` - The cell background color.
///
/// # Errors
///
/// Return a Lua argument error when the supplied value is not a supported color expression.
pub(super) fn parse_color(
  value: Value,
  method: &str,
  name: &str,
  background: bool,
) -> mlua::Result<Option<TextColor>> {
  if matches!(value, Value::Nil) {
    return Ok(None);
  }
  let value = args::string(value, method, name)?;
  if value == "none" {
    return Ok(None);
  }
  if value == "transparent" {
    if background {
      return Ok(Some(TextColor::Transparent));
    }
    return Err(args::message(
      method,
      "transparent is only valid for background colors",
    ));
  }
  parse_text_color(&value)
    .map(Some)
    .ok_or_else(|| args::message(method, format!("invalid color '{value}'")))
}

/// Read an optional positive terminal-cell dimension that fits in `u16`.
///
/// # Arguments
///
/// * `table` - The Lua table to inspect or convert.
/// * `method` - The script-visible method name included in argument errors.
/// * `name` - The name used to identify the object or field.
///
/// # Errors
///
/// Return a Lua argument error when a supplied dimension is non-integral, non-positive, or
/// exceeds `u16`.
pub(super) fn optional_positive_u16(
  table: &Table,
  method: &str,
  name: &str,
) -> mlua::Result<Option<u16>> {
  let value = args::optional_integer(table, method, name, None)?;
  value
    .map(|value| {
      u16::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| args::message(method, format!("{name} must be in 1..=65535")))
    })
    .transpose()
}

/// Validate a positive terminal-cell dimension that fits in `u16`.
///
/// # Arguments
///
/// * `table` - The Lua table to inspect or convert.
/// * `method` - The script-visible method name included in argument errors.
/// * `name` - The name used to identify the object or field.
///
/// # Errors
///
/// Return a Lua argument error when the dimension is non-integral, non-positive, or exceeds
/// `u16`.
pub(super) fn positive_u16(table: &Table, method: &str, name: &str) -> mlua::Result<u16> {
  let value = args::integer(args::required(table, method, name)?, method, name)?;
  u16::try_from(value)
    .ok()
    .filter(|value| *value > 0)
    .ok_or_else(|| args::message(method, format!("{name} must be in 1..=65535")))
}

/// Resolve an optional Lua drawing destination into a base, slice, or scroll-box target.
///
/// # Arguments
///
/// * `table` - The Lua table to inspect or convert.
/// * `method` - The script-visible method name included in argument errors.
/// * `state` - The state.
///
/// # Errors
///
/// Return a Lua argument error when the target kind or identifier is invalid or does not belong
/// to the session.
pub(super) fn parse_draw_target(
  table: &Table,
  method: &str,
  state: &SharedApiState,
) -> mlua::Result<LuaDrawTarget> {
  let layer = args::optional_string(table, method, "slice_layer", Some("base"))?.unwrap();
  if layer == "base" {
    return Ok(LuaDrawTarget::Base);
  }
  let id = slice::parse_id(&layer, method, "slice_layer")?;
  let objects = state
    .borrow()
    .objects
    .upgrade()
    .ok_or_else(|| args::message(method, "session object pool is unavailable"))?;
  let objects = objects.borrow();
  let pool = objects
    .as_ref()
    .ok_or_else(|| args::message(method, "session object pool is unavailable"))?;
  if tg_service_widget::SliceService::new().exists(pool.ui(), id) {
    Ok(LuaDrawTarget::Slice(id))
  } else {
    Err(args::message(
      method,
      format!("unknown or inaccessible slice layer '{layer}'"),
    ))
  }
}

/// Return the available terminal-cell dimensions of the validated drawing target.
///
/// # Arguments
///
/// * `state` - The state.
/// * `method` - The script-visible method name included in argument errors.
/// * `target` - The object or resource affected by the operation.
///
/// # Errors
///
/// Return a Lua argument error when the requested drawing target cannot be resolved in the
/// session pool.
///
/// # Panics
///
/// Panic when passed a scroll-box target; callers must supply the base or a session-owned slice.
pub(super) fn draw_target_size(
  state: &SharedApiState,
  method: &str,
  target: LuaDrawTarget,
) -> mlua::Result<tg_service_layout::Size> {
  if target == LuaDrawTarget::Base {
    return Ok(state.borrow().context.base_size);
  }
  let LuaDrawTarget::Slice(id) = target else {
    unreachable!();
  };
  let base = state.borrow().context.base_size;
  let objects = state
    .borrow()
    .objects
    .upgrade()
    .ok_or_else(|| args::message(method, "session object pool is unavailable"))?;
  let objects = objects.borrow();
  let pool = objects
    .as_ref()
    .ok_or_else(|| args::message(method, "session object pool is unavailable"))?;
  let rect = tg_service_widget::SliceService::new()
    .configured_rect(pool.ui(), id)
    .ok_or_else(|| args::message(method, "unknown or inaccessible slice layer"))?;
  Ok(tg_service_layout::Size {
    width: slice::resolve_length(rect.width, base.width),
    height: slice::resolve_length(rect.height, base.height),
  })
}
