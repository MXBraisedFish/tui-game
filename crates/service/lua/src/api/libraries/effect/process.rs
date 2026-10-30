//! Validate cell callback input, partial output, and independent coordinate updates.

use super::super::*;
use crate::LuaEffectRequest;
use tg_core_style::TextStyle;
use tg_service_canvas::{CellEffectCell, CellEffectRegion, buffer::CanvasBuffer};

const METHOD: &str = "effect.process";
const FIELDS: &[&str] = &[
  "char",
  "x",
  "y",
  "relative_x",
  "relative_y",
  "fg",
  "bg",
  "bold",
  "italic",
  "underline",
  "strike",
  "blink",
  "reverse",
  "hidden",
  "dim",
];

pub(crate) fn process_buffer(
  lua: &Lua,
  request: &LuaEffectRequest,
  buffer: &mut CanvasBuffer,
  offset: (i32, i32),
) -> mlua::Result<()> {
  let region = CellEffectRegion {
    x: request.x.saturating_sub(offset.0),
    y: request.y.saturating_sub(offset.1),
    width: request.effect.width,
    height: request.effect.height,
    include_continuation: request.effect.include_continuation,
  };
  buffer.transform_region(region, |original| {
    let x = original.x + offset.0;
    let y = original.y + offset.1;
    let relative_x = i64::from(x) - i64::from(request.x);
    let relative_y = i64::from(y) - i64::from(request.y);
    let input = lua.create_table()?;
    input.raw_set("char", original.cell.text.as_str())?;
    for (name, value) in [
      ("x", i64::from(x)),
      ("y", i64::from(y)),
      ("relative_x", relative_x),
      ("relative_y", relative_y),
    ] {
      input.raw_set(name, value)?;
    }
    input.raw_set(
      "fg",
      slice::background_name(original.cell.style.foreground.clone()),
    )?;
    input.raw_set(
      "bg",
      slice::background_name(original.cell.style.background.clone()),
    )?;
    for (name, value) in flags(&original.cell.style) {
      input.raw_set(name, value)?;
    }
    let returned = request.effect.process.call::<Value>(input)?;
    let Value::Table(output) = returned else {
      return Err(args::invalid(METHOD, "return value", "table", &returned));
    };
    let output = positional_table(
      lua,
      METHOD,
      MultiValue::from_vec(vec![Value::Table(output)]),
      &[],
      FIELDS,
    )?;
    let new_x = position(&output, "x", "relative_x", x, relative_x, request.x)?;
    let new_y = position(&output, "y", "relative_y", y, relative_y, request.y)?;
    let mut cell = original.cell.clone();
    if !matches!(output.raw_get::<Value>("char")?, Value::Nil) {
      cell.text = args::string(output.raw_get("char")?, METHOD, "char")?;
      let graphemes = tg_core_unicode::graphemes(&cell.text);
      if !cell.text.is_empty()
        && (cell.text.chars().any(char::is_control)
          || graphemes.len() != 1
          || graphemes[0].display_width == 0)
      {
        return Err(args::message(
          METHOD,
          "char must be empty or contain one visible grapheme",
        ));
      }
    }
    for (name, stored, background) in [
      ("fg", &mut cell.style.foreground, false),
      ("bg", &mut cell.style.background, true),
    ] {
      let value = output.raw_get::<Value>(name)?;
      if !matches!(value, Value::Nil) {
        let text = args::string(value.clone(), METHOD, name)?;
        if text != slice::background_name(stored.clone()) {
          *stored = parse_color(value, METHOD, name, background)?;
        }
      }
    }
    cell.style.bold = args::optional_bool(&output, METHOD, "bold", cell.style.bold)?;
    cell.style.italic = args::optional_bool(&output, METHOD, "italic", cell.style.italic)?;
    cell.style.underline = args::optional_bool(&output, METHOD, "underline", cell.style.underline)?;
    cell.style.strike = args::optional_bool(&output, METHOD, "strike", cell.style.strike)?;
    cell.style.blink = args::optional_bool(&output, METHOD, "blink", cell.style.blink)?;
    cell.style.reverse = args::optional_bool(&output, METHOD, "reverse", cell.style.reverse)?;
    cell.style.hidden = args::optional_bool(&output, METHOD, "hidden", cell.style.hidden)?;
    cell.style.dim = args::optional_bool(&output, METHOD, "dim", cell.style.dim)?;
    // Transparent unwritten cells must not become opaque simply by passing through a shader.
    let written = original.written || !cell.text.is_empty() || cell.style != TextStyle::default();
    Ok(CellEffectCell {
      x: new_x.saturating_sub(offset.0),
      y: new_y.saturating_sub(offset.1),
      cell,
      written,
    })
  })
}

fn position(
  table: &Table,
  absolute: &str,
  relative: &str,
  original: i32,
  original_relative: i64,
  origin: i32,
) -> mlua::Result<i32> {
  let absolute_value = table.raw_get::<Value>(absolute)?;
  let relative_value = table.raw_get::<Value>(relative)?;
  let absolute_value = if matches!(absolute_value, Value::Nil) {
    i64::from(original)
  } else {
    checked_coordinate(absolute_value, absolute)?
  };
  let relative_value = if matches!(relative_value, Value::Nil) {
    original_relative
  } else {
    checked_coordinate(relative_value, relative)?
  };
  let position = if absolute_value != i64::from(original) {
    absolute_value
  } else if relative_value != original_relative {
    i64::from(origin) + relative_value
  } else {
    i64::from(original)
  };
  i32::try_from(position)
    .map_err(|_| args::message(METHOD, format!("resolved {absolute} is out of i32 range")))
}

fn checked_coordinate(value: Value, name: &str) -> mlua::Result<i64> {
  let value = args::integer(value, METHOD, name)?;
  i32::try_from(value).map_err(|_| args::message(METHOD, format!("{name} is out of i32 range")))?;
  Ok(value)
}

fn flags(style: &TextStyle) -> [(&'static str, bool); 8] {
  [
    ("bold", style.bold),
    ("italic", style.italic),
    ("underline", style.underline),
    ("strike", style.strike),
    ("blink", style.blink),
    ("reverse", style.reverse),
    ("hidden", style.hidden),
    ("dim", style.dim),
  ]
}
