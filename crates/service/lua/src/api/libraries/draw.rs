use super::*;

pub(super) fn draw(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  let text_state = state.clone();
  source.raw_set(
    "text",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "draw.text";
      let table = draw_text_parameters(lua, method, values)?;
      let target = parse_draw_target(&table, method, &text_state)?;
      let params = {
        let state = text_state.borrow();
        parse_draw_text_params(&table, method, &state.context, true)?
      };
      let x = args::integer(args::required(&table, method, "x")?, method, "x")?;
      let y = args::integer(args::required(&table, method, "y")?, method, "y")?;
      enqueue_draw(
        &text_state,
        method,
        LuaDrawCommand::Text {
          target,
          x: checked_i32(x, method, "x")?,
          y: checked_i32(y, method, "y")?,
          params,
        },
      )
    })?,
  )?;
  let fill_state = state.clone();
  source.raw_set(
    "fill_rect",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "draw.fill_rect";
      let table = positional_table(
        lua,
        method,
        values,
        &["x", "y", "width", "height"],
        &["char", "fg", "bg", "slice_layer"],
      )?;
      let target = parse_draw_target(&table, method, &fill_state)?;
      let fill_char = optional_single_char(&table, method, "char")?;
      let command = LuaDrawCommand::FillRect {
        target,
        x: signed_coordinate(&table, method, "x")?,
        y: signed_coordinate(&table, method, "y")?,
        width: positive_u16(&table, method, "width")?,
        height: positive_u16(&table, method, "height")?,
        fill_char,
        fg: parse_color(table.get::<Value>("fg")?, method, "fg", false)?,
        bg: parse_color(table.get::<Value>("bg")?, method, "bg", true)?,
      };
      enqueue_draw(&fill_state, method, command)
    })?,
  )?;
  let stroke_state = state.clone();
  source.raw_set(
    "stroke_rect",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "draw.stroke_rect";
      let table = positional_table(
        lua,
        method,
        values,
        &["x", "y", "width", "height"],
        &["fg", "bg", "border_char", "slice_layer"],
      )?;
      let target = parse_draw_target(&table, method, &stroke_state)?;
      let command = LuaDrawCommand::StrokeRect {
        target,
        x: signed_coordinate(&table, method, "x")?,
        y: signed_coordinate(&table, method, "y")?,
        width: positive_u16(&table, method, "width")?,
        height: positive_u16(&table, method, "height")?,
        border: parse_border(table.get::<Value>("border_char")?, method)?,
        fg: parse_color(table.get::<Value>("fg")?, method, "fg", false)?,
        bg: parse_color(table.get::<Value>("bg")?, method, "bg", true)?,
      };
      enqueue_draw(&stroke_state, method, command)
    })?,
  )?;
  let erase_state = state.clone();
  source.raw_set(
    "erase_rect",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "draw.erase_rect";
      let table = positional_table(
        lua,
        method,
        values,
        &["x", "y", "width", "height"],
        &["slice_layer"],
      )?;
      let target = parse_draw_target(&table, method, &erase_state)?;
      let command = LuaDrawCommand::EraseRect {
        target,
        x: signed_coordinate(&table, method, "x")?,
        y: signed_coordinate(&table, method, "y")?,
        width: positive_u16(&table, method, "width")?,
        height: positive_u16(&table, method, "height")?,
      };
      enqueue_draw(&erase_state, method, command)
    })?,
  )?;
  let state2 = state;
  source.raw_set(
    "render",
    lua.create_function(move |_, values: MultiValue| {
      args::no_args("draw.render", values)?;
      let mut state = state2.borrow_mut();
      if state.phase == LuaCallPhase::Render {
        return Err(args::message(
          "draw.render",
          "invalid_state: draw.render cannot be called during Render",
        ));
      }
      if !state
        .commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::RequestRender))
      {
        push_host_command(&mut state, LuaHostCommand::RequestRender);
      }
      Ok(())
    })?,
  )?;
  readonly::proxy(lua, source)
}

fn enqueue_draw(state: &SharedApiState, method: &str, command: LuaDrawCommand) -> mlua::Result<()> {
  let mut state = state.borrow_mut();
  if state.draw_command_count >= 4096 {
    state.fatal_api_error = true;
    return Err(args::message(
      method,
      "draw command buffer exceeds 4096 commands",
    ));
  }
  if let LuaDrawCommand::Text { params, .. } = &command {
    state.draw_text_bytes = state.draw_text_bytes.saturating_add(params.text.len());
    if state.draw_text_bytes > args::MAX_API_STRING_BYTES {
      state.fatal_api_error = true;
      return Err(args::message(
        method,
        "draw text exceeds 1 MiB in one frame",
      ));
    }
  }
  state.draw_command_count += 1;
  push_host_command(&mut state, LuaHostCommand::Draw(command));
  Ok(())
}

fn signed_coordinate(table: &Table, method: &str, name: &str) -> mlua::Result<i32> {
  checked_i32(
    args::integer(args::required(table, method, name)?, method, name)?,
    method,
    name,
  )
}

fn checked_i32(value: i64, method: &str, name: &str) -> mlua::Result<i32> {
  i32::try_from(value).map_err(|_| args::message(method, format!("{name} is out of i32 range")))
}

fn optional_single_char(table: &Table, method: &str, name: &str) -> mlua::Result<Option<String>> {
  let value = table.get::<Value>(name)?;
  if matches!(value, Value::Nil) {
    return Ok(None);
  }
  let text = args::string(value, method, name)?;
  let mut chars = text.chars();
  let Some(ch) = chars.next() else {
    return Err(args::message(
      method,
      format!("{name} must contain one display cell"),
    ));
  };
  if chars.next().is_some() || tg_core_unicode::char_width(ch) != 1 {
    return Err(args::message(
      method,
      format!("{name} must contain one display-cell character"),
    ));
  }
  Ok(Some(ch.to_string()))
}

fn parse_border(value: Value, method: &str) -> mlua::Result<BorderStyle> {
  if matches!(value, Value::Nil) {
    return Ok(BorderStyle::Line);
  }
  let Value::Table(table) = value else {
    return Err(args::invalid(method, "border_char", "table or nil", &value));
  };
  let table = readonly::backing(&table)?;
  let get = |name: &str| -> mlua::Result<BorderCharacter> {
    let value = table.get::<Value>(name)?;
    if matches!(value, Value::Nil) {
      return Ok(BorderCharacter::default());
    }
    let text = args::string(value, method, "border_char")?;
    let mut chars = text.chars();
    let Some(ch) = chars.next() else {
      return Ok(BorderCharacter::default());
    };
    if chars.next().is_some() || tg_core_unicode::char_width(ch) != 1 {
      return Err(args::message(
        method,
        "border characters must occupy one display cell",
      ));
    }
    Ok(BorderCharacter {
      char: Some(ch),
      ..Default::default()
    })
  };
  Ok(BorderStyle::Custom(CustomBorder {
    top: get("top")?,
    left_top: get("left_top")?,
    left: get("left")?,
    left_bottom: get("left_bottom")?,
    bottom: get("bottom")?,
    right_bottom: get("right_bottom")?,
    right: get("right")?,
    right_top: get("right_top")?,
  }))
}
