use super::*;

pub(super) fn utf8(lua: &Lua) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  source.raw_set(
    "len",
    single_text(
      lua,
      "utf8.len",
      |s| Value::Integer(s.chars().count() as i64),
    )?,
  )?;
  source.raw_set(
    "byte_len",
    single_text(lua, "utf8.byte_len", |s| Value::Integer(s.len() as i64))?,
  )?;
  source.raw_set(
    "is_ascii",
    single_text(lua, "utf8.is_ascii", |s| Value::Boolean(s.is_ascii()))?,
  )?;
  source.raw_set(
    "codepoint_to_char",
    lua.create_function(|lua, values: MultiValue| {
      let value = args::one("utf8.codepoint_to_char", "values", values)?;
      let values = args::array_values(value, "utf8.codepoint_to_char", "values")?;
      let mut output = String::new();
      for value in values {
        let cp = args::integer(value, "utf8.codepoint_to_char", "values")?;
        let Some(ch) = char::from_u32(cp.try_into().unwrap_or(u32::MAX)) else {
          return Err(args::message(
            "utf8.codepoint_to_char",
            "invalid Unicode codepoint",
          ));
        };
        output.push(ch);
        if output.len() > args::MAX_API_STRING_BYTES {
          return Err(args::message(
            "utf8.codepoint_to_char",
            "output exceeds 1 MiB",
          ));
        }
      }
      lua.create_string(output)
    })?,
  )?;
  source.raw_set(
    "ascii_to_char",
    lua.create_function(|lua, values: MultiValue| {
      let value = args::one("utf8.ascii_to_char", "values", values)?;
      let values = args::array_values(value, "utf8.ascii_to_char", "values")?;
      let mut output = Vec::with_capacity(values.len());
      for value in values {
        let value = args::integer(value, "utf8.ascii_to_char", "values")?;
        if !(0..=127).contains(&value) {
          return Err(args::message(
            "utf8.ascii_to_char",
            "ASCII values must be in 0..=127",
          ));
        }
        output.push(value as u8);
      }
      lua.create_string(output)
    })?,
  )?;
  source.raw_set(
    "char_to_codepoint",
    lua.create_function(|lua, values: MultiValue| utf8_codes(lua, values, false))?,
  )?;
  source.raw_set(
    "char_to_ascii",
    lua.create_function(|lua, values: MultiValue| utf8_codes(lua, values, true))?,
  )?;
  source.raw_set(
    "char_position",
    lua.create_function(|lua, values: MultiValue| {
      let parameters = args::positional(
        lua,
        "utf8.char_position",
        values,
        &["text", "index"],
        &["start"],
      )?;
      let text = args::string(
        parameters.required(0, "utf8.char_position", "text")?,
        "utf8.char_position",
        "text",
      )?;
      let index = args::integer(
        parameters.required(1, "utf8.char_position", "index")?,
        "utf8.char_position",
        "index",
      )?;
      if index < 1 {
        return Err(args::message(
          "utf8.char_position",
          "index must be at least 1",
        ));
      }
      let start =
        args::optional_integer(parameters.options(), "utf8.char_position", "start", Some(1))?
          .unwrap();
      let length = text.chars().count();
      let start = resolve_index(start, length, true);
      let target = start.and_then(|start| {
        let value = start as i128 + index as i128 - 1;
        (value >= 0 && value < length as i128).then_some(value as usize)
      });
      let position =
        target.and_then(|index| text.char_indices().nth(index).map(|(byte, _)| byte + 1));
      Ok(position.map_or(Value::Nil, |position| Value::Integer(position as i64)))
    })?,
  )?;
  source.raw_set(
    "codepoints",
    lua.create_function(|lua, values: MultiValue| {
      let text = args::lua_string(
        args::one("utf8.codepoints", "text", values)?,
        "utf8.codepoints",
        "text",
      )?;
      let byte_offset = std::rc::Rc::new(std::cell::Cell::new(0_usize));
      lua.create_function(move |_, _: MultiValue| {
        let text = text
          .to_str()
          .map_err(|_| args::message("utf8.codepoints", "text must be valid UTF-8"))?;
        let text = text.as_ref();
        let offset = byte_offset.get();
        let Some(character) = text[offset..].chars().next() else {
          return Ok(MultiValue::from_vec(vec![Value::Nil]));
        };
        byte_offset.set(offset + character.len_utf8());
        Ok(MultiValue::from_vec(vec![
          Value::Integer((offset + 1) as i64),
          Value::Integer(character as u32 as i64),
        ]))
      })
    })?,
  )?;
  source.raw_set(
    "next",
    lua.create_function(|lua, values: MultiValue| {
      let parameters = args::positional(lua, "utf8.next", values, &["text"], &["pos"])?;
      let text = args::lua_string(
        parameters.required(0, "utf8.next", "text")?,
        "utf8.next",
        "text",
      )?;
      let text = text
        .to_str()
        .map_err(|_| args::message("utf8.next", "text must be valid UTF-8"))?;
      let text = text.as_ref();
      let pos = args::optional_integer(parameters.options(), "utf8.next", "pos", None)?;
      if pos.is_some_and(|position| position < 1) {
        return Err(args::message("utf8.next", "pos must be at least 1"));
      }
      let start = match pos {
        Some(position) => {
          let Ok(position) = usize::try_from(position) else {
            return Ok(MultiValue::from_vec(vec![Value::Nil]));
          };
          if position > text.len() {
            return Ok(MultiValue::from_vec(vec![Value::Nil]));
          }
          let mut boundary = position;
          while boundary < text.len() && !text.is_char_boundary(boundary) {
            boundary += 1;
          }
          boundary
        }
        None => 0,
      };
      let found = text[start..]
        .char_indices()
        .next()
        .map(|(offset, character)| (start + offset, character));
      let Some((position, character)) = found else {
        return Ok(MultiValue::from_vec(vec![Value::Nil]));
      };
      Ok(MultiValue::from_vec(vec![
        Value::Integer((position + 1) as i64),
        Value::Integer(character as u32 as i64),
      ]))
    })?,
  )?;
  readonly::proxy(lua, source)
}

pub(super) fn single_text(
  lua: &Lua,
  method: &'static str,
  operation: fn(&str) -> Value,
) -> mlua::Result<Function> {
  lua.create_function(move |_, values: MultiValue| {
    let text = args::string(args::one(method, "text", values)?, method, "text")?;
    Ok(operation(&text))
  })
}
pub(super) fn resolve_index(index: i64, len: usize, allow_end: bool) -> Option<usize> {
  if len == 0 && !allow_end {
    return None;
  }
  let value = if index < 0 {
    len as i64 + index
  } else {
    index - 1
  };
  let max = if allow_end {
    len
  } else {
    len.saturating_sub(1)
  };
  (value >= 0 && (value as usize) <= max).then_some(value as usize)
}
pub(super) fn utf8_codes(lua: &Lua, values: MultiValue, ascii_only: bool) -> mlua::Result<Table> {
  let method = if ascii_only {
    "utf8.char_to_ascii"
  } else {
    "utf8.char_to_codepoint"
  };
  let parameters = args::positional(lua, method, values, &["text"], &["start", "finish"])?;
  let text = args::string(parameters.required(0, method, "text")?, method, "text")?;
  let options = parameters.options();
  let chars = text.chars().collect::<Vec<_>>();
  let start = args::optional_integer(options, method, "start", Some(1))?.unwrap();
  let default_finish = chars.len() as i64;
  let finish = args::optional_integer(options, method, "finish", Some(default_finish))?.unwrap();
  let output = lua.create_table()?;
  let Some(start) = resolve_index(start, chars.len(), false) else {
    output.raw_set("n", 0)?;
    return Ok(output);
  };
  let Some(finish) = resolve_index(finish, chars.len(), false) else {
    output.raw_set("n", 0)?;
    return Ok(output);
  };
  if finish < start {
    output.raw_set("n", 0)?;
    return Ok(output);
  }
  let count = finish - start + 1;
  if count > args::MAX_API_TABLE_ENTRIES {
    return Err(args::message(method, "result exceeds 16384 entries"));
  }
  for (index, ch) in chars[start..=finish].iter().copied().enumerate() {
    if !ascii_only || ch.is_ascii() {
      output.raw_set(index + 1, ch as u32)?;
    }
  }
  output.raw_set("n", count)?;
  Ok(output)
}
