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
      lua.create_function(move |_, values: MultiValue| {
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
        let table = args::named(
          method,
          values,
          &[
            dimension,
            align_name,
            offset_name,
            relative_name,
            "slice_layer",
          ],
        )?;
        let target = parse_draw_target(&table, method, &state)?;
        let size = args::integer(
          args::required(&table, method, dimension)?,
          method,
          dimension,
        )?;
        if size <= 0 {
          return Err(args::message(
            method,
            format!("{dimension} must be positive"),
          ));
        }
        let align = args::string(
          args::required(&table, method, align_name)?,
          method,
          align_name,
        )?;
        let offset = args::optional_integer(&table, method, offset_name, Some(0))?.unwrap();
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
          args::optional_integer(&table, method, relative_name, None)?,
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
      let table = args::named(
        method,
        values,
        &[
          "width",
          "height",
          "horizontal_align",
          "vertical_align",
          "offset_x",
          "offset_y",
          "relative_x",
          "relative_y",
          "slice_layer",
        ],
      )?;
      let target = parse_draw_target(&table, method, &state)?;
      let width = positive_u16(&table, method, "width")? as i64;
      let height = positive_u16(&table, method, "height")? as i64;
      let horizontal = args::string(
        args::required(&table, method, "horizontal_align")?,
        method,
        "horizontal_align",
      )?;
      let vertical = args::string(
        args::required(&table, method, "vertical_align")?,
        method,
        "vertical_align",
      )?;
      let target_size = draw_target_size(&state, method, target)?;
      let x = resolve_alignment_axis(
        method,
        width,
        target_size.width as i64,
        &horizontal,
        args::optional_integer(&table, method, "relative_x", None)?,
        args::optional_integer(&table, method, "offset_x", Some(0))?.unwrap(),
        true,
      )?;
      let y = resolve_alignment_axis(
        method,
        height,
        target_size.height as i64,
        &vertical,
        args::optional_integer(&table, method, "relative_y", None)?,
        args::optional_integer(&table, method, "offset_y", Some(0))?.unwrap(),
        false,
      )?;
      let result = lua.create_table()?;
      result.raw_set("x", x)?;
      result.raw_set("y", y)?;
      Ok(result)
    })?,
  )?;
  readonly::proxy(lua, source)
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
