use std::cmp::Ordering;
use std::f64::consts::{E, PI};
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::Duration;

use mlua::{Function, Lua, MultiValue, Table, Value};
use regex::{Regex, RegexBuilder};

use super::args;
use super::readonly;
use super::{
  LuaApiContext, LuaCallPhase, LuaDrawCommand, LuaDrawTarget, LuaHostCommand, SharedApiState,
};
use crate::host_engine::services::lua::LuaSessionKind;
use crate::host_engine::services::{
  BorderCharacter, BorderStyle, CustomBorder, DrawTextParams, FileTask, LuaFileOperation,
  TextAlign, TextColor, TextMode, TextWrapMode, parse_text_color,
};

mod align;
mod base;
#[path = "libraries/char.rs"]
mod chars;
mod color;
mod debug;
mod draw;
mod encoding;
mod event;
mod file;
mod game;
mod i18n;
mod loader;
mod math;
mod measurement;
mod random;
mod serialization;
mod slice;
mod string;
mod table;
mod utf8;

use measurement::{
  draw_target_size, draw_text_parameters, parse_color, parse_draw_target, parse_draw_text_params,
  positive_u16,
};
use string::rich_text_params;

const MAX_HOST_COMMANDS_PER_CALLBACK: usize = 4096;

pub fn install(lua: &Lua, environment: &Table, state: SharedApiState) -> mlua::Result<()> {
  let base = base::base(lua)?;
  environment.set("base", base.clone())?;
  for name in [
    "ipairs",
    "pairs",
    "next",
    "select",
    "rawequal",
    "rawlen",
    "tonumber",
    "tostring",
    "type",
    "setmetatable",
    "getmetatable",
  ] {
    environment.set(name, base.get::<Value>(name)?)?;
  }
  environment.set("math", math::math(lua)?)?;
  environment.set("utf8", utf8::utf8(lua)?)?;
  environment.set("table", table::table_lib(lua)?)?;
  environment.set("string", string::string_lib(lua, state.clone())?)?;
  environment.set("color", color::color(lua)?)?;
  environment.set("char", chars::char_lib(lua)?)?;
  environment.set("align", align::align(lua, state.clone())?)?;
  environment.set("measurement", measurement::measurement(lua, state.clone())?)?;
  environment.set("random", random::random(lua, state.clone())?)?;
  environment.set("slice", slice::slice(lua, state.clone())?)?;
  environment.set("serialization", serialization::serialization(lua)?)?;
  environment.set("encoding", encoding::encoding(lua)?)?;
  environment.set("draw", draw::draw(lua, state.clone())?)?;
  environment.set("debug", debug::debug(lua, state.clone())?)?;
  environment.set("game", game::game(lua, state.clone())?)?;
  environment.set("i18n", i18n::i18n(lua, state.clone())?)?;
  environment.set("event", event::event(lua, state.clone())?)?;
  environment.set("loader", loader::loader(lua, environment, state.clone())?)?;
  environment.set("file", file::file(lua, state)?)?;
  Ok(())
}

fn function_value(function: Function) -> Value {
  Value::Function(function)
}

fn string_value(lua: &Lua, value: &str) -> mlua::Result<Value> {
  Ok(Value::String(lua.create_string(value)?))
}

fn ignore_once(state: &mut super::LuaApiState, method: &'static str, reason: &'static str) {
  if state.ignored_methods.insert(method) {
    push_host_command(state, LuaHostCommand::Ignored { method, reason });
  }
}

fn push_host_command(state: &mut super::LuaApiState, command: LuaHostCommand) {
  if state.commands.len() >= MAX_HOST_COMMANDS_PER_CALLBACK {
    state.fatal_api_error = true;
  } else {
    state.commands.push(command);
  }
}

fn enqueue_debug_print(
  state: &mut super::LuaApiState,
  message: String,
  title: Option<String>,
  time: bool,
  level: Option<String>,
  type_head: bool,
) {
  if state.debug_log_window_started.elapsed() >= Duration::from_secs(1) {
    if state.debug_log_dropped > 0 {
      push_host_command(
        state,
        LuaHostCommand::Log {
          level: "warn".to_string(),
          message: format!(
            "suppressed {} Lua debug log messages due to rate limiting",
            state.debug_log_dropped
          ),
        },
      );
    }
    state.debug_log_window_started = std::time::Instant::now();
    state.debug_log_count = 0;
    state.debug_log_dropped = 0;
  }
  if state.debug_log_count < 100 {
    state.debug_log_count += 1;
    push_host_command(
      state,
      LuaHostCommand::Print {
        message: truncate(message, 4096),
        title: title.map(|title| truncate(title, 4096)),
        time,
        level,
        type_head,
      },
    );
  } else {
    state.debug_log_dropped = state.debug_log_dropped.saturating_add(1);
  }
}

fn truncate(mut value: String, max: usize) -> String {
  if value.len() > max {
    while !value.is_char_boundary(max.min(value.len())) {
      value.pop();
    }
    value.truncate(max);
  }
  value
}
