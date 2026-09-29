use super::*;

mod pattern;

use pattern::{LuaCapture, LuaCaptures, LuaPattern, LuaPatternInput, MAX_CAPTURE_GROUPS};

pub(super) fn string_lib(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, value) in [
    ("AUTO", "auto"),
    ("PLAIN_TEXT", "plain_text"),
    ("RICH_TEXT", "rich_text"),
  ] {
    source.raw_set(name, value)?;
  }
  for (name, operation) in [
    ("lower", StringUnary::Lower),
    ("upper", StringUnary::Upper),
    ("reverse", StringUnary::Reverse),
    ("regex_escape", StringUnary::RegexEscape),
  ] {
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method = format!("string.{name}");
        let text = args::string(args::one(&method, "text", values)?, &method, "text")?;
        let output = operation.apply(&text);
        ensure_output_size(&method, &output)?;
        Ok(output)
      })?,
    )?;
  }
  source.raw_set(
    "split",
    lua.create_function(|lua, values: MultiValue| {
      let method = "string.split";
      let parameters = args::positional(lua, method, values, &["text", "sep"], &[])?;
      let text = args::string(parameters.required(0, method, "text")?, method, "text")?;
      let separator = args::string(parameters.required(1, method, "sep")?, method, "sep")?;
      if separator.is_empty() {
        return Err(args::message(method, "sep must not be empty"));
      }
      let output = lua.create_table()?;
      for (index, part) in text.split(&separator).enumerate() {
        if index >= 10_000 {
          return Err(args::message(method, "result exceeds 10000 items"));
        }
        output.raw_set(index + 1, part)?;
      }
      Ok(output)
    })?,
  )?;
  source.raw_set(
    "sub",
    lua.create_function(|lua, values: MultiValue| {
      let parameters =
        args::positional(lua, "string.sub", values, &["text", "start"], &["finish"])?;
      let options = parameters.options();
      let text = args::string(
        parameters.required(0, "string.sub", "text")?,
        "string.sub",
        "text",
      )?;
      let chars = text.chars().collect::<Vec<_>>();
      let start = args::integer(
        parameters.required(1, "string.sub", "start")?,
        "string.sub",
        "start",
      )?;
      let finish =
        args::optional_integer(options, "string.sub", "finish", Some(chars.len() as i64))?.unwrap();
      let start = relative_sub_index(start, chars.len()).max(1);
      let finish = relative_sub_index(finish, chars.len()).min(chars.len() as i128);
      if start > finish {
        return Ok(String::new());
      }
      Ok(chars[start as usize - 1..finish as usize].iter().collect())
    })?,
  )?;
  source.raw_set(
    "rep",
    lua.create_function(|lua, values: MultiValue| {
      let parameters = args::positional(lua, "string.rep", values, &["text", "times"], &["sep"])?;
      let options = parameters.options();
      let text = args::string(
        parameters.required(0, "string.rep", "text")?,
        "string.rep",
        "text",
      )?;
      let times = args::integer(
        parameters.required(1, "string.rep", "times")?,
        "string.rep",
        "times",
      )?;
      let sep = args::optional_string(options, "string.rep", "sep", Some(""))?.unwrap();
      if times < 0 {
        return Err(args::message("string.rep", "times must be non-negative"));
      }
      let times =
        usize::try_from(times).map_err(|_| args::message("string.rep", "times is too large"))?;
      let size = text
        .len()
        .checked_mul(times)
        .and_then(|v| v.checked_add(sep.len().saturating_mul(times.saturating_sub(1))))
        .ok_or_else(|| args::message("string.rep", "output size overflow"))?;
      if size > args::MAX_API_STRING_BYTES {
        return Err(args::message("string.rep", "output exceeds 1 MiB"));
      }
      if size == 0 {
        return Ok(String::new());
      }
      let mut output = String::with_capacity(size);
      for index in 0..times {
        if index > 0 {
          output.push_str(&sep);
        }
        output.push_str(&text);
      }
      Ok(output)
    })?,
  )?;
  source.raw_set("find", string_find(lua, false)?)?;
  source.raw_set("match", string_match(lua, false)?)?;
  source.raw_set("gmatch", string_gmatch(lua, false)?)?;
  source.raw_set("gsub", string_gsub(lua, false)?)?;
  source.raw_set("regex_find", string_find(lua, true)?)?;
  source.raw_set("regex_match", string_match(lua, true)?)?;
  source.raw_set("regex_gmatch", string_gmatch(lua, true)?)?;
  source.raw_set("regex_gsub", string_gsub(lua, true)?)?;
  source.raw_set(
    "regex_test",
    lua.create_function(|lua, values: MultiValue| {
      let parameters =
        args::positional(lua, "string.regex_test", values, &["text", "pattern"], &[])?;
      let text = args::string(
        parameters.required(0, "string.regex_test", "text")?,
        "string.regex_test",
        "text",
      )?;
      let pattern = pattern_parameter(parameters.get(1), "string.regex_test", true)?;
      Ok(
        pattern
          .captures(&text, 0)
          .map_err(|message| args::message("string.regex_test", message))?
          .is_some(),
      )
    })?,
  )?;
  source.raw_set(
    "regex_split",
    lua.create_function(|lua, values: MultiValue| {
      let parameters =
        args::positional(lua, "string.regex_split", values, &["text", "pattern"], &[])?;
      let text = args::string(
        parameters.required(0, "string.regex_split", "text")?,
        "string.regex_split",
        "text",
      )?;
      let pattern = pattern_parameter(parameters.get(1), "string.regex_split", true)?;
      let output = lua.create_table()?;
      let mut total = 0_usize;
      for (index, part) in pattern
        .split(&text)
        .map_err(|message| args::message("string.regex_split", message))?
        .into_iter()
        .take(10_001)
        .enumerate()
      {
        if index == 10_000 {
          return Err(args::message(
            "string.regex_split",
            "result exceeds 10000 items",
          ));
        }
        total = total.saturating_add(part.len());
        if total > args::MAX_API_STRING_BYTES {
          return Err(args::message("string.regex_split", "output exceeds 1 MiB"));
        }
        output.raw_set(index + 1, part)?;
      }
      Ok(output)
    })?,
  )?;
  source.raw_set(
    "format",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::variadic("string.format", values, &["format_string"])?;
      let format_string = args::string(parameters[0].clone(), "string.format", "format_string")?;
      safe_format(&format_string, &parameters[1..])
    })?,
  )?;
  let rich_text_state = state;
  source.raw_set(
    "rich_text_to_plain_text",
    lua.create_function(move |lua, values: MultiValue| {
      let parameters = args::positional(
        lua,
        "string.rich_text_to_plain_text",
        values,
        &["text"],
        &["rich_params", "key_params", "strip_header"],
      )?;
      let options = parameters.options();
      let text = args::string(
        parameters.required(0, "string.rich_text_to_plain_text", "text")?,
        "string.rich_text_to_plain_text",
        "text",
      )?;
      let key_params = args::optional_bool(
        options,
        "string.rich_text_to_plain_text",
        "key_params",
        true,
      )?;
      let strip_header = args::optional_bool(
        options,
        "string.rich_text_to_plain_text",
        "strip_header",
        true,
      )?;
      let mut params = rich_text_params(
        options.get::<Value>("rich_params")?,
        "string.rich_text_to_plain_text",
      )?;
      if key_params && (text.contains("{key:") || text.contains("{key_default:")) {
        let context = &rich_text_state.borrow().context;
        let params = params.get_or_insert_with(Default::default);
        if text.contains("{key:") {
          params.key_actions.clone_from(&context.key_actions);
        }
        if text.contains("{key_default:") {
          params
            .key_default_actions
            .clone_from(&context.key_default_actions);
        }
      }
      let input = if strip_header {
        text.strip_prefix("f%").unwrap_or(&text)
      } else {
        &text
      };
      let output = tg_service_rich_text::RichTextService::new()
        .visible_text(&format!("f%{input}"), params.as_ref());
      ensure_output_size("string.rich_text_to_plain_text", &output)?;
      Ok(output)
    })?,
  )?;
  readonly::proxy(lua, source)
}

fn relative_sub_index(index: i64, len: usize) -> i128 {
  if index >= 0 {
    index as i128
  } else {
    len as i128 + index as i128 + 1
  }
}

fn ensure_output_size(method: &str, output: &str) -> mlua::Result<()> {
  if output.len() > args::MAX_API_STRING_BYTES {
    Err(args::message(method, "output exceeds 1 MiB"))
  } else {
    Ok(())
  }
}

enum CompiledPattern {
  Lua(LuaPattern),
  Regex(Regex),
}

#[derive(Clone)]
enum PatternCapture {
  Text(Range<usize>),
  Position(usize),
}

#[derive(Clone)]
struct PatternCaptures {
  full: Range<usize>,
  captures: Vec<Option<PatternCapture>>,
}

impl PatternCaptures {
  fn len(&self) -> usize {
    self.captures.len() + 1
  }

  fn value(&self, index: usize) -> Option<PatternCapture> {
    if index == 0 {
      Some(PatternCapture::Text(self.full.clone()))
    } else {
      self.captures.get(index - 1).cloned().flatten()
    }
  }
}

impl From<LuaCaptures> for PatternCaptures {
  fn from(value: LuaCaptures) -> Self {
    Self {
      full: value.full,
      captures: value
        .captures
        .into_iter()
        .map(|capture| {
          capture.map(|capture| match capture {
            LuaCapture::Text(range) => PatternCapture::Text(range),
            LuaCapture::Position(position) => PatternCapture::Position(position),
          })
        })
        .collect(),
    }
  }
}

impl CompiledPattern {
  fn captures(&self, text: &str, offset: usize) -> Result<Option<PatternCaptures>, String> {
    match self {
      Self::Lua(pattern) => pattern
        .captures(text, text[..offset].chars().count())
        .map(|capture| capture.map(Into::into)),
      Self::Regex(pattern) => Ok(pattern.captures_at(text, offset).map(|captures| {
        let full = captures.get(0).unwrap();
        PatternCaptures {
          full: full.start()..full.end(),
          captures: (1..captures.len())
            .map(|index| {
              captures
                .get(index)
                .map(|capture| PatternCapture::Text(capture.start()..capture.end()))
            })
            .collect(),
        }
      })),
    }
  }

  fn captures_iter(&self, text: &str) -> Result<Vec<PatternCaptures>, String> {
    match self {
      Self::Lua(pattern) => pattern
        .captures_iter(text)
        .map(|captures| captures.into_iter().map(Into::into).collect()),
      Self::Regex(pattern) => Ok(
        pattern
          .captures_iter(text)
          .take(10_001)
          .map(|captures| {
            let full = captures.get(0).unwrap();
            PatternCaptures {
              full: full.start()..full.end(),
              captures: (1..captures.len())
                .map(|index| {
                  captures
                    .get(index)
                    .map(|capture| PatternCapture::Text(capture.start()..capture.end()))
                })
                .collect(),
            }
          })
          .collect(),
      ),
    }
  }

  fn next_captures(
    &self,
    text: &str,
    byte_start: usize,
    char_start: usize,
    lua_input: Option<&LuaPatternInput>,
    lua_pattern_steps: &mut usize,
  ) -> Result<Option<PatternCaptures>, String> {
    match self {
      Self::Lua(pattern) => {
        let input = lua_input.ok_or_else(|| "Lua pattern input is unavailable".to_string())?;
        pattern
          .captures_incremental_with_input(text, input, char_start, lua_pattern_steps)
          .map(|captures| captures.map(Into::into))
      }
      Self::Regex(pattern) => Ok(pattern.captures_at(text, byte_start).map(|captures| {
        let full = captures.get(0).unwrap();
        PatternCaptures {
          full: full.start()..full.end(),
          captures: (1..captures.len())
            .map(|index| {
              captures
                .get(index)
                .map(|capture| PatternCapture::Text(capture.start()..capture.end()))
            })
            .collect(),
        }
      })),
    }
  }

  fn captures_iter_limited(
    &self,
    text: &str,
    limit: usize,
  ) -> Result<Vec<PatternCaptures>, String> {
    match self {
      Self::Lua(pattern) => pattern
        .captures_iter_limited(text, limit)
        .map(|captures| captures.into_iter().map(Into::into).collect()),
      Self::Regex(pattern) => Ok(
        pattern
          .captures_iter(text)
          .take(limit)
          .map(|captures| {
            let full = captures.get(0).unwrap();
            PatternCaptures {
              full: full.start()..full.end(),
              captures: (1..captures.len())
                .map(|index| {
                  captures
                    .get(index)
                    .map(|capture| PatternCapture::Text(capture.start()..capture.end()))
                })
                .collect(),
            }
          })
          .collect(),
      ),
    }
  }

  fn split(&self, text: &str) -> Result<Vec<String>, String> {
    let captures = self.captures_iter(text)?;
    let mut output = Vec::with_capacity(captures.len() + 1);
    let mut last = 0;
    for capture in captures {
      output.push(text[last..capture.full.start].to_string());
      last = capture.full.end;
    }
    output.push(text[last..].to_string());
    Ok(output)
  }
}

fn pattern_parameter(value: Value, method: &str, regex: bool) -> mlua::Result<CompiledPattern> {
  let pattern = args::string(value, method, "pattern")?;
  compile_pattern(&pattern, method, regex)
}

fn compile_pattern(pattern: &str, method: &str, regex: bool) -> mlua::Result<CompiledPattern> {
  validate_pattern_size(pattern, method)?;
  if regex {
    let pattern = RegexBuilder::new(pattern)
      .size_limit(1024 * 1024)
      .build()
      .map_err(|error| args::message(method, format!("invalid pattern: {error}")))?;
    if pattern.captures_len() > MAX_CAPTURE_GROUPS + 1 {
      return Err(args::message(
        method,
        format!("pattern exceeds {MAX_CAPTURE_GROUPS} captures"),
      ));
    }
    Ok(CompiledPattern::Regex(pattern))
  } else {
    LuaPattern::compile(pattern)
      .map(CompiledPattern::Lua)
      .map_err(|message| args::message(method, format!("invalid pattern: {message}")))
  }
}

fn validate_pattern_size(pattern: &str, method: &str) -> mlua::Result<()> {
  if pattern.len() > 8 * 1024 {
    Err(args::message(method, "pattern exceeds 8 KiB"))
  } else {
    Ok(())
  }
}

fn search_start(text: &str, index: i64) -> Option<usize> {
  let chars = text
    .char_indices()
    .map(|(index, _)| index)
    .collect::<Vec<_>>();
  let relative = if index >= 0 {
    index as i128
  } else {
    chars.len() as i128 + index as i128 + 1
  };
  let one_based = relative.max(1);
  if one_based > chars.len() as i128 + 1 {
    return None;
  }
  chars
    .get(one_based as usize - 1)
    .copied()
    .or(Some(text.len()))
}

fn char_position(text: &str, byte: usize) -> i64 {
  text[..byte].chars().count() as i64 + 1
}

fn string_find(lua: &Lua, regex_mode: bool) -> mlua::Result<Function> {
  lua.create_function(move |lua, values: MultiValue| {
    let method = if regex_mode {
      "string.regex_find"
    } else {
      "string.find"
    };
    let options = if regex_mode {
      &["init"][..]
    } else {
      &["init", "plain"][..]
    };
    let parameters = args::positional(lua, method, values, &["text", "pattern"], options)?;
    let options = parameters.options();
    let text = args::string(parameters.required(0, method, "text")?, method, "text")?;
    let pattern = args::string(parameters.get(1), method, "pattern")?;
    let init = args::optional_integer(options, method, "init", Some(1))?.unwrap();
    let plain = !regex_mode && args::optional_bool(options, method, "plain", false)?;
    if !plain {
      validate_pattern_size(&pattern, method)?;
    }
    let Some(offset) = search_start(&text, init) else {
      return Ok(MultiValue::from_vec(vec![Value::Nil]));
    };
    if plain {
      let Some(found) = text[offset..].find(&pattern) else {
        return Ok(MultiValue::from_vec(vec![Value::Nil]));
      };
      let start = offset + found;
      let finish = start + pattern.len();
      let captures = lua.create_table()?;
      captures.raw_set(1, &text[start..finish])?;
      captures.raw_set("n", 1)?;
      return find_result(&text, start, finish, captures);
    }
    let pattern = compile_pattern(&pattern, method, regex_mode)?;
    let Some(captures) = pattern
      .captures(&text, offset)
      .map_err(|message| args::message(method, message))?
    else {
      return Ok(MultiValue::from_vec(vec![Value::Nil]));
    };
    let start = captures.full.start;
    let finish = captures.full.end;
    let values = capture_values_table(lua, method, &text, &captures)?;
    find_result(&text, start, finish, values)
  })
}

fn string_match(lua: &Lua, regex_mode: bool) -> mlua::Result<Function> {
  lua.create_function(move |lua, values: MultiValue| {
    let method = if regex_mode {
      "string.regex_match"
    } else {
      "string.match"
    };
    let parameters = args::positional(lua, method, values, &["text", "pattern"], &["init"])?;
    let text = args::string(parameters.required(0, method, "text")?, method, "text")?;
    let pattern = args::string(parameters.get(1), method, "pattern")?;
    let init = args::optional_integer(parameters.options(), method, "init", Some(1))?.unwrap();
    validate_pattern_size(&pattern, method)?;
    let Some(offset) = search_start(&text, init) else {
      return Ok(Value::Nil);
    };
    let pattern = compile_pattern(&pattern, method, regex_mode)?;
    let Some(captures) = pattern
      .captures(&text, offset)
      .map_err(|message| args::message(method, message))?
    else {
      return Ok(Value::Nil);
    };
    capture_values_table(lua, method, &text, &captures).map(Value::Table)
  })
}

fn string_gmatch(lua: &Lua, regex_mode: bool) -> mlua::Result<Function> {
  lua.create_function(move |lua, values: MultiValue| {
    let method = if regex_mode {
      "string.regex_gmatch"
    } else {
      "string.gmatch"
    };
    let parameters = args::positional(lua, method, values, &["text", "pattern"], &[])?;
    let text = args::lua_string(parameters.required(0, method, "text")?, method, "text")?;
    let pattern = pattern_parameter(parameters.get(1), method, regex_mode)?;
    let state = std::rc::Rc::new(std::cell::RefCell::new(GmatchState::default()));
    lua.create_function(move |lua, _: MultiValue| {
      let mut state = state.borrow_mut();
      if state.exhausted {
        return Ok(Value::Nil);
      }
      let text = text
        .to_str()
        .map_err(|_| args::message(method, "text must be valid UTF-8"))?;
      let text = text.as_ref();
      if matches!(&pattern, CompiledPattern::Lua(_)) && state.lua_pattern_input.is_none() {
        state.lua_pattern_input = Some(LuaPatternInput::new(text));
      }
      let captures_result = {
        let GmatchState {
          byte_start,
          char_start,
          lua_pattern_input,
          lua_pattern_steps,
          ..
        } = &mut *state;
        pattern.next_captures(
          text,
          *byte_start,
          *char_start,
          lua_pattern_input.as_ref(),
          lua_pattern_steps,
        )
      };
      let captures = match captures_result {
        Ok(Some(captures)) => captures,
        Ok(None) => {
          state.exhausted = true;
          state.lua_pattern_input = None;
          return Ok(Value::Nil);
        }
        Err(message) => {
          state.exhausted = true;
          state.lua_pattern_input = None;
          return Err(args::message(method, message));
        }
      };
      if state.matches >= 10_000 {
        state.exhausted = true;
        state.lua_pattern_input = None;
        return Err(args::message(method, "result exceeds 10000 items"));
      }
      let bytes = match capture_output_bytes(method, &captures) {
        Ok(bytes) => bytes,
        Err(error) => {
          state.exhausted = true;
          state.lua_pattern_input = None;
          return Err(error);
        }
      };
      state.output_bytes = state
        .output_bytes
        .checked_add(bytes)
        .ok_or_else(|| args::message(method, "result size overflow"))?;
      if state.output_bytes > args::MAX_API_STRING_BYTES {
        state.exhausted = true;
        state.lua_pattern_input = None;
        return Err(args::message(method, "result exceeds 1 MiB"));
      }
      state.matches += 1;
      let previous_byte_start = state.byte_start;
      if captures.full.end > captures.full.start {
        state.byte_start = captures.full.end;
      } else if captures.full.end < text.len() {
        state.byte_start = captures.full.end
          + text[captures.full.end..]
            .chars()
            .next()
            .map_or(0, char::len_utf8);
      } else {
        state.exhausted = true;
      }
      state.char_start += text[previous_byte_start..state.byte_start].chars().count();
      if state.exhausted {
        state.lua_pattern_input = None;
      }
      capture_values_table(lua, method, text, &captures).map(Value::Table)
    })
  })
}

#[derive(Default)]
struct GmatchState {
  byte_start: usize,
  char_start: usize,
  lua_pattern_input: Option<LuaPatternInput>,
  matches: usize,
  output_bytes: usize,
  lua_pattern_steps: usize,
  exhausted: bool,
}

fn string_gsub(lua: &Lua, regex_mode: bool) -> mlua::Result<Function> {
  lua.create_function(move |lua, values: MultiValue| {
    let method = if regex_mode {
      "string.regex_gsub"
    } else {
      "string.gsub"
    };
    let parameters = args::positional(
      lua,
      method,
      values,
      &["text", "pattern", "repl"],
      &["limit"],
    )?;
    let text = args::string(parameters.required(0, method, "text")?, method, "text")?;
    let pattern = pattern_parameter(parameters.get(1), method, regex_mode)?;
    let replacement = parameters.required(2, method, "repl")?;
    if !matches!(
      &replacement,
      Value::String(_) | Value::Table(_) | Value::Function(_)
    ) {
      return Err(args::invalid(
        method,
        "repl",
        "string, table, or function",
        &replacement,
      ));
    }
    let limit = args::optional_integer(parameters.options(), method, "limit", Some(-1))?.unwrap();
    if limit < -1 {
      return Err(args::message(method, "limit must be -1 or non-negative"));
    }
    if limit > 10_000 {
      return Err(args::message(method, "limit must not exceed 10000"));
    }
    let unlimited = limit == -1;
    let limit = if unlimited { 10_000 } else { limit as usize };
    if limit == 0 {
      return Ok(MultiValue::from_vec(vec![
        Value::String(lua.create_string(text)?),
        Value::Integer(0),
      ]));
    }
    let mut result = String::with_capacity(text.len());
    let mut last = 0_usize;
    let mut count = 0_usize;
    let captures_list = pattern
      .captures_iter_limited(&text, limit + usize::from(unlimited))
      .map_err(|message| args::message(method, message))?;
    if unlimited && captures_list.len() > 10_000 {
      return Err(args::message(method, "result exceeds 10000 matches"));
    }
    for captures in captures_list {
      if count >= limit {
        break;
      }
      result.push_str(&text[last..captures.full.start]);
      let value = replacement_value(lua, method, &replacement, &captures, &text, regex_mode)?;
      result.push_str(value.as_deref().unwrap_or(&text[captures.full.clone()]));
      if result.len() > args::MAX_API_STRING_BYTES {
        return Err(args::message(method, "output exceeds 1 MiB"));
      }
      last = captures.full.end;
      count += 1;
    }
    result.push_str(&text[last..]);
    if result.len() > args::MAX_API_STRING_BYTES {
      return Err(args::message(method, "output exceeds 1 MiB"));
    }
    Ok(MultiValue::from_vec(vec![
      Value::String(lua.create_string(result)?),
      Value::Integer(count as i64),
    ]))
  })
}

fn find_result(
  text: &str,
  start: usize,
  finish: usize,
  captures: Table,
) -> mlua::Result<MultiValue> {
  Ok(MultiValue::from_vec(vec![
    Value::Integer(char_position(text, start)),
    Value::Integer(text[..finish].chars().count() as i64),
    Value::Table(captures),
  ]))
}

fn capture_values_table(
  lua: &Lua,
  method: &str,
  text: &str,
  captures: &PatternCaptures,
) -> mlua::Result<Table> {
  let first = if captures.len() > 1 { 1 } else { 0 };
  let count = captures.len() - first;
  let output = lua.create_table()?;
  output.raw_set("n", count)?;
  let mut output_bytes = 0_usize;
  for (output_index, capture_index) in (first..captures.len()).enumerate() {
    if let Some(capture) = captures.value(capture_index) {
      output_bytes = output_bytes
        .checked_add(match &capture {
          PatternCapture::Text(range) => range.len(),
          PatternCapture::Position(position) => position.to_string().len(),
        })
        .ok_or_else(|| args::message(method, "result size overflow"))?;
      if output_bytes > args::MAX_API_STRING_BYTES {
        return Err(args::message(method, "result exceeds 1 MiB"));
      }
      output.raw_set(output_index + 1, pattern_capture_value(lua, text, capture)?)?;
    }
  }
  Ok(output)
}

fn capture_output_bytes(method: &str, captures: &PatternCaptures) -> mlua::Result<usize> {
  let first = if captures.len() > 1 { 1 } else { 0 };
  (first..captures.len()).try_fold(0_usize, |total, index| {
    let bytes = match captures.value(index) {
      Some(PatternCapture::Text(range)) => range.len(),
      Some(PatternCapture::Position(position)) => position.to_string().len(),
      None => 0,
    };
    total
      .checked_add(bytes)
      .ok_or_else(|| args::message(method, "result size overflow"))
  })
}

fn replacement_value(
  lua: &Lua,
  method: &str,
  replacement: &Value,
  captures: &PatternCaptures,
  text: &str,
  regex_mode: bool,
) -> mlua::Result<Option<String>> {
  let key = captures.value(1).or_else(|| captures.value(0)).unwrap();
  let key = pattern_capture_value(lua, text, key)?;
  let value = match replacement {
    Value::String(value) => {
      let source = value.to_str()?;
      let mut output = String::new();
      let mut chars = source.chars().peekable();
      while let Some(ch) = chars.next() {
        let marker = if regex_mode { '$' } else { '%' };
        if ch == marker {
          match chars.peek().copied() {
            Some(next @ '0'..='9') => {
              chars.next();
              let index = next.to_digit(10).unwrap() as usize;
              match captures.value(index) {
                Some(value) => output.push_str(&pattern_capture_text(text, value)),
                None if index < captures.len() => {}
                None => {
                  return Err(args::message(
                    method,
                    format!("replacement references missing capture {index}"),
                  ));
                }
              }
            }
            Some(next) if next == marker => {
              chars.next();
              output.push(marker);
            }
            Some(next) => {
              return Err(args::message(
                method,
                format!("invalid replacement escape '{marker}{next}'"),
              ));
            }
            None => {
              return Err(args::message(
                method,
                "replacement ends with an escape marker",
              ));
            }
          }
        } else {
          output.push(ch);
        }
      }
      Some(output)
    }
    Value::Table(table) => lua_value_to_replacement(method, table.get::<Value>(key)?)?,
    Value::Function(function) => {
      let first = if captures.len() > 1 { 1 } else { 0 };
      let parameters = (first..captures.len())
        .map(|index| match captures.value(index) {
          Some(value) => pattern_capture_value(lua, text, value),
          None => Ok(Value::Nil),
        })
        .collect::<mlua::Result<Vec<_>>>()?;
      lua_value_to_replacement(
        method,
        function.call::<Value>(MultiValue::from_vec(parameters))?,
      )?
    }
    value => {
      return Err(args::invalid(
        method,
        "repl",
        "string, table, or function",
        value,
      ));
    }
  };
  Ok(value)
}

fn pattern_capture_text(text: &str, capture: PatternCapture) -> String {
  match capture {
    PatternCapture::Text(range) => text[range].to_string(),
    PatternCapture::Position(position) => position.to_string(),
  }
}

fn pattern_capture_value(lua: &Lua, text: &str, capture: PatternCapture) -> mlua::Result<Value> {
  match capture {
    PatternCapture::Text(range) => lua.create_string(&text[range]).map(Value::String),
    PatternCapture::Position(position) => Ok(Value::Integer(position as i64)),
  }
}

fn lua_value_to_replacement(method: &str, value: Value) -> mlua::Result<Option<String>> {
  match value {
    Value::String(value) => value
      .to_str()
      .map(|value| Some(value.to_string()))
      .map_err(|_| args::message(method, "replacement returned invalid UTF-8")),
    Value::Integer(value) => Ok(Some(value.to_string())),
    Value::Number(value) => Ok(Some(value.to_string())),
    Value::Boolean(false) | Value::Nil => Ok(None),
    value => Err(args::message(
      method,
      format!(
        "replacement must resolve to string, integer, float, false, or nil, got {}",
        args::type_name(&value)
      ),
    )),
  }
}

fn safe_format(format_string: &str, values: &[Value]) -> mlua::Result<String> {
  let mut output = String::new();
  let mut chars = format_string.chars().peekable();
  let mut index = 0_usize;
  while let Some(ch) = chars.next() {
    if ch != '%' {
      output.push(ch);
      continue;
    }
    if chars.peek() == Some(&'%') {
      chars.next();
      output.push('%');
      continue;
    }
    let spec = FormatSpec::parse(&mut chars)?;
    let Some(kind) = chars.next() else {
      return Err(args::message(
        "string.format",
        "incomplete format specifier",
      ));
    };
    let value = values
      .get(index)
      .ok_or_else(|| args::message("string.format", "not enough values"))?;
    index += 1;
    let (formatted, numeric) = match kind {
      's' => {
        let mut value = format_value(value)?;
        if let Some(precision) = spec.precision {
          value = value.chars().take(precision).collect();
        }
        (value, false)
      }
      'q' => {
        if spec.has_modifiers() {
          return Err(args::message(
            "string.format",
            "format '%q' does not accept flags, width, or precision",
          ));
        }
        (format_quoted(&format_value(value)?), false)
      }
      'd' | 'i' => {
        let value = args::integer(value.clone(), "string.format", "values")?;
        (format_signed_integer(value, &spec), true)
      }
      'u' => {
        let value = args::integer(value.clone(), "string.format", "values")? as u64;
        (format_unsigned_integer(value, 10, false, &spec), true)
      }
      'x' | 'X' => {
        let value = args::integer(value.clone(), "string.format", "values")? as u64;
        (format_unsigned_integer(value, 16, kind == 'X', &spec), true)
      }
      'o' => {
        let value = args::integer(value.clone(), "string.format", "values")? as u64;
        (format_unsigned_integer(value, 8, false, &spec), true)
      }
      'f' | 'e' | 'E' | 'g' | 'G' => {
        let value = args::number(value.clone(), "string.format", "values")?;
        let precision = spec.precision.unwrap_or(6).min(32);
        let value = match kind {
          'f' => format_fixed(value, precision, spec.alternate),
          'e' | 'E' => format_scientific(value, precision, kind == 'E', spec.alternate),
          'g' | 'G' => format_general(value, precision.max(1), kind == 'G', spec.alternate),
          _ => unreachable!("numeric format branch only handles float conversions"),
        };
        let mut value = value;
        if !value.starts_with('-') {
          if spec.plus {
            value.insert(0, '+');
          } else if spec.space {
            value.insert(0, ' ');
          }
        }
        (value, true)
      }
      'c' => {
        let value = char::from_u32(
          u32::try_from(args::integer(value.clone(), "string.format", "values")?)
            .map_err(|_| args::message("string.format", "character code is out of range"))?,
        )
        .ok_or_else(|| args::message("string.format", "invalid Unicode scalar value"))?
        .to_string();
        (value, false)
      }
      _ => {
        return Err(args::message(
          "string.format",
          format!("unsupported format '%{kind}'"),
        ));
      }
    };
    let formatted = apply_format_width(formatted, &spec, numeric);
    output.push_str(&formatted);
    if output.len() > args::MAX_API_STRING_BYTES {
      return Err(args::message("string.format", "output exceeds 1 MiB"));
    }
  }
  ensure_output_size("string.format", &output)?;
  Ok(output)
}

#[derive(Default)]
struct FormatSpec {
  left: bool,
  plus: bool,
  space: bool,
  alternate: bool,
  zero: bool,
  width: Option<usize>,
  precision: Option<usize>,
}

impl FormatSpec {
  fn parse<I>(chars: &mut std::iter::Peekable<I>) -> mlua::Result<Self>
  where
    I: Iterator<Item = char>,
  {
    let mut output = Self::default();
    loop {
      match chars.peek().copied() {
        Some('-') => output.left = true,
        Some('+') => output.plus = true,
        Some(' ') => output.space = true,
        Some('#') => output.alternate = true,
        Some('0') => output.zero = true,
        _ => break,
      }
      chars.next();
    }
    output.width = parse_format_number(chars)?;
    if chars.peek() == Some(&'.') {
      chars.next();
      output.precision = Some(parse_format_number(chars)?.unwrap_or(0));
    }
    if output
      .width
      .is_some_and(|width| width > args::MAX_API_STRING_BYTES)
    {
      return Err(args::message("string.format", "format width exceeds 1 MiB"));
    }
    Ok(output)
  }

  fn has_modifiers(&self) -> bool {
    self.left
      || self.plus
      || self.space
      || self.alternate
      || self.zero
      || self.width.is_some()
      || self.precision.is_some()
  }
}

fn parse_format_number<I>(chars: &mut std::iter::Peekable<I>) -> mlua::Result<Option<usize>>
where
  I: Iterator<Item = char>,
{
  let mut value = None::<usize>;
  while let Some(digit) = chars.peek().and_then(|value| value.to_digit(10)) {
    chars.next();
    value = Some(
      value
        .unwrap_or(0)
        .checked_mul(10)
        .and_then(|value| value.checked_add(digit as usize))
        .ok_or_else(|| args::message("string.format", "format width is too large"))?,
    );
  }
  Ok(value)
}

fn format_signed_integer(value: i64, spec: &FormatSpec) -> String {
  let negative = value < 0;
  let precision = spec.precision.map(|precision| precision.min(32));
  let digits = padded_integer_digits(value.unsigned_abs(), precision, 10, false);
  let sign = if negative {
    "-"
  } else if spec.plus {
    "+"
  } else if spec.space {
    " "
  } else {
    ""
  };
  format!("{sign}{digits}")
}

fn format_unsigned_integer(value: u64, radix: u32, uppercase: bool, spec: &FormatSpec) -> String {
  let precision = spec.precision.map(|precision| precision.min(32));
  let digits = padded_integer_digits(value, precision, radix, uppercase);
  let prefix = if spec.alternate {
    match (radix, uppercase) {
      (8, _) if !digits.starts_with('0') => "0",
      (16, false) if value != 0 => "0x",
      (16, true) if value != 0 => "0X",
      _ => "",
    }
  } else {
    ""
  };
  format!("{prefix}{digits}")
}

fn format_fixed(value: f64, precision: usize, alternate: bool) -> String {
  if !value.is_finite() {
    return format_special_float(value, false);
  }
  let mut output = format!("{value:.precision$}");
  if alternate && !output.contains('.') {
    output.push('.');
  }
  output
}

fn format_quoted(value: &str) -> String {
  let bytes = value.as_bytes();
  let mut output = String::with_capacity(value.len() + 2);
  output.push('"');
  let mut copied_until = 0;

  for (index, character) in value.char_indices() {
    let escape = match character {
      '"' => Some("\\\"".to_string()),
      '\\' => Some("\\\\".to_string()),
      '\n' => Some("\\\n".to_string()),
      character if character.is_ascii_control() => {
        let code = character as u8;
        let followed_by_digit = bytes.get(index + 1).is_some_and(u8::is_ascii_digit);
        Some(if followed_by_digit {
          format!("\\{code:03}")
        } else {
          format!("\\{code}")
        })
      }
      _ => None,
    };
    if let Some(escape) = escape {
      output.push_str(&value[copied_until..index]);
      output.push_str(&escape);
      copied_until = index + character.len_utf8();
    }
  }

  output.push_str(&value[copied_until..]);
  output.push('"');
  output
}

fn format_scientific(value: f64, precision: usize, uppercase: bool, alternate: bool) -> String {
  if !value.is_finite() {
    return format_special_float(value, uppercase);
  }
  let output = format!("{:.*e}", precision, value);
  let (mantissa, exponent) = output.split_once('e').unwrap();
  let exponent = exponent.parse::<i32>().unwrap();
  let mantissa = ensure_decimal_point(mantissa.to_string(), alternate);
  format!(
    "{mantissa}{}{:+03}",
    if uppercase { 'E' } else { 'e' },
    exponent
  )
}

fn format_general(value: f64, precision: usize, uppercase: bool, alternate: bool) -> String {
  if !value.is_finite() {
    return format_special_float(value, uppercase);
  }

  let scientific = format!("{:.*e}", precision.saturating_sub(1), value.abs());
  let (mantissa, exponent) = scientific.split_once('e').unwrap();
  let exponent = exponent.parse::<i32>().unwrap();
  let use_scientific = exponent < -4 || exponent >= precision as i32;
  let mut output = if use_scientific {
    let mantissa = ensure_decimal_point(mantissa.to_string(), alternate);
    format!(
      "{mantissa}{}{:+03}",
      if uppercase { 'E' } else { 'e' },
      exponent
    )
  } else {
    let decimals = (precision as i32 - exponent - 1).max(0) as usize;
    format!("{:.*}", decimals, value.abs())
  };

  if !alternate {
    trim_general_fraction(&mut output);
  } else if !output.contains('.') {
    output.push('.');
  }
  if value.is_sign_negative() {
    output.insert(0, '-');
  }
  output
}

fn format_special_float(value: f64, uppercase: bool) -> String {
  let word = if value.is_nan() {
    if uppercase { "NAN" } else { "nan" }
  } else if uppercase {
    "INF"
  } else {
    "inf"
  };
  if value.is_sign_negative() && !value.is_nan() {
    format!("-{word}")
  } else {
    word.to_string()
  }
}

fn ensure_decimal_point(mut value: String, alternate: bool) -> String {
  if alternate && !value.contains('.') {
    value.push('.');
  }
  value
}

fn trim_general_fraction(value: &mut String) {
  let exponent_start = value.find(['e', 'E']).unwrap_or(value.len());
  let Some(decimal_point) = value[..exponent_start].find('.') else {
    return;
  };
  let mut end = exponent_start;
  while end > decimal_point + 1 && value.as_bytes()[end - 1] == b'0' {
    end -= 1;
  }
  if end == decimal_point + 1 {
    end -= 1;
  }
  if end < exponent_start {
    value.replace_range(end..exponent_start, "");
  }
}

fn padded_integer_digits(
  value: u64,
  precision: Option<usize>,
  radix: u32,
  uppercase: bool,
) -> String {
  let mut digits = match (radix, uppercase) {
    (8, _) => format!("{value:o}"),
    (16, false) => format!("{value:x}"),
    (16, true) => format!("{value:X}"),
    _ => value.to_string(),
  };
  if precision == Some(0) && value == 0 {
    digits.clear();
  }
  if let Some(precision) = precision
    && digits.len() < precision
  {
    digits.insert_str(0, &"0".repeat(precision - digits.len()));
  }
  digits
}

fn apply_format_width(mut value: String, spec: &FormatSpec, numeric: bool) -> String {
  let width = spec.width.unwrap_or(0);
  let length = value.chars().count();
  if length >= width {
    return value;
  }
  let padding = width - length;
  if spec.left {
    value.push_str(&" ".repeat(padding));
  } else if numeric && spec.zero && spec.precision.is_none() {
    let prefix = if value.starts_with('+') || value.starts_with('-') || value.starts_with(' ') {
      1
    } else if value.starts_with("0x") || value.starts_with("0X") {
      2
    } else {
      0
    };
    value.insert_str(prefix, &"0".repeat(padding));
  } else {
    value.insert_str(0, &" ".repeat(padding));
  }
  value
}

fn format_value(value: &Value) -> mlua::Result<String> {
  args::dynamic_text(value.clone(), "string.format", "values")
}

pub(super) fn rich_text_params(
  value: Value,
  method: &str,
) -> mlua::Result<Option<tg_service_rich_text::RichTextParams>> {
  if matches!(value, Value::Nil) {
    return Ok(None);
  }
  let Value::Table(table) = value else {
    return Err(args::invalid(method, "rich_params", "table or nil", &value));
  };
  let mut output = tg_service_rich_text::RichTextParams::default();
  for pair in table.pairs::<String, Value>() {
    let (key, value) = pair?;
    if key.len() > args::MAX_API_STRING_BYTES {
      return Err(args::message(method, "rich_params key exceeds 1 MiB"));
    }
    let value = match value {
      Value::String(value) => value.to_str()?.to_string(),
      Value::Integer(value) => value.to_string(),
      Value::Number(value) if value.is_finite() => value.to_string(),
      Value::Boolean(value) => value.to_string(),
      value => {
        return Err(args::invalid(
          method,
          "rich_params",
          "string, integer, float, or boolean values",
          &value,
        ));
      }
    };
    if value.len() > args::MAX_API_STRING_BYTES {
      return Err(args::message(method, "rich_params value exceeds 1 MiB"));
    }
    output.values.insert(key, value);
  }
  Ok(Some(output))
}
#[derive(Clone, Copy)]
enum StringUnary {
  Lower,
  Upper,
  Reverse,
  RegexEscape,
}
impl StringUnary {
  fn apply(self, text: &str) -> String {
    match self {
      Self::Lower => text.to_lowercase(),
      Self::Upper => text.to_uppercase(),
      Self::Reverse => text.chars().rev().collect(),
      Self::RegexEscape => regex::escape(text),
    }
  }
}
