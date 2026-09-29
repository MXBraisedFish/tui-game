use super::*;

pub(super) fn align(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, value) in [
    ("AUTO", "auto"),
    ("LEFT", "left"),
    ("HORIZONTAL_CENTER", "horizontal_center"),
    ("RIGHT", "right"),
    ("TOP", "top"),
    ("VERTICAL_CENTER", "vertical_center"),
    ("BOTTOM", "bottom"),
    ("CENTER", "center"),
  ] {
    source.raw_set(name, value)?;
  }
  for (name, axis) in [("resolve_x", 0_u8), ("resolve_y", 1_u8)] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let method = if axis == 0 {
          "align.resolve_x"
        } else {
          "align.resolve_y"
        };
        let dimension = if axis == 0 { "width" } else { "height" };
        let align_name = if axis == 0 {
          "horizontal_align"
        } else {
          "vertical_align"
        };
        let offset_name = if axis == 0 { "offset_x" } else { "offset_y" };
        let relative_name = if axis == 0 {
          "relative_x"
        } else {
          "relative_y"
        };
        let parameters = args::positional(
          lua,
          method,
          values,
          &[dimension, align_name],
          &[offset_name, relative_name, "slice_layer"],
        )?;
        let options = parameters.options();
        let target = parse_draw_target(options, method, &state)?;
        let size = positive_dimension(parameters.get(0), method, dimension)?;
        let align = args::string(parameters.get(1), method, align_name)?;
        let offset = args::optional_integer(options, method, offset_name, Some(0))?.unwrap();
        let target_size = draw_target_size(&state, method, target)?;
        let available = if axis == 0 {
          target_size.width
        } else {
          target_size.height
        } as i64;
        resolve_alignment_axis(
          method,
          size,
          available,
          &align,
          args::optional_integer(options, method, relative_name, None)?,
          offset,
          axis == 0,
        )
      })?,
    )?;
  }
  let state = state.clone();
  source.raw_set(
    "resolve_rect",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "align.resolve_rect";
      let parameters = args::positional(
        lua,
        method,
        values,
        &["width", "height", "horizontal_align", "vertical_align"],
        &[
          "offset_x",
          "offset_y",
          "relative_x",
          "relative_y",
          "slice_layer",
        ],
      )?;
      let options = parameters.options();
      let target = parse_draw_target(options, method, &state)?;
      let width = positive_dimension(parameters.get(0), method, "width")?;
      let height = positive_dimension(parameters.get(1), method, "height")?;
      let horizontal = args::string(parameters.get(2), method, "horizontal_align")?;
      let vertical = args::string(parameters.get(3), method, "vertical_align")?;
      let target_size = draw_target_size(&state, method, target)?;
      let x = resolve_alignment_axis(
        method,
        width,
        target_size.width as i64,
        &horizontal,
        args::optional_integer(options, method, "relative_x", None)?,
        args::optional_integer(options, method, "offset_x", Some(0))?.unwrap(),
        true,
      )?;
      let y = resolve_alignment_axis(
        method,
        height,
        target_size.height as i64,
        &vertical,
        args::optional_integer(options, method, "relative_y", None)?,
        args::optional_integer(options, method, "offset_y", Some(0))?.unwrap(),
        false,
      )?;
      Ok(MultiValue::from_vec(vec![
        Value::Integer(x),
        Value::Integer(y),
      ]))
    })?,
  )?;
  readonly::proxy(lua, source)
}

fn positive_dimension(value: Value, method: &str, name: &str) -> mlua::Result<i64> {
  let value = args::integer(value, method, name)?;
  if (1..=65535).contains(&value) {
    Ok(value)
  } else {
    Err(args::message(
      method,
      format!("{name} must be in 1..=65535"),
    ))
  }
}

fn resolve_alignment_axis(
  method: &str,
  size: i64,
  available: i64,
  alignment: &str,
  relative: Option<i64>,
  offset: i64,
  horizontal: bool,
) -> mlua::Result<i64> {
  let start = if horizontal { "left" } else { "top" };
  let center = if horizontal {
    "horizontal_center"
  } else {
    "vertical_center"
  };
  let end = if horizontal { "right" } else { "bottom" };
  let anchor = relative.unwrap_or_else(|| match alignment {
    value if value == start => 0,
    value if value == center || value == "center" || value == "auto" => available / 2,
    value if value == end => available,
    _ => 0,
  });
  let result = match alignment {
    value if value == start => i128::from(anchor) + i128::from(offset),
    value if value == center || value == "center" || value == "auto" => {
      i128::from(anchor) - i128::from(size) / 2 + i128::from(offset)
    }
    value if value == end => i128::from(anchor) - i128::from(size) + i128::from(offset),
    _ => return Err(args::message(method, "invalid alignment constant")),
  };
  i64::try_from(result).map_err(|_| args::message(method, "resolved coordinate is out of range"))
}
