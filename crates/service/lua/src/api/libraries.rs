//! Construction and registration of the supported Lua API libraries.

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
use crate::LuaFileOperation;
use crate::LuaSessionKind;
use tg_core_style::{TextColor, parse_text_color};
use tg_service_file::FileTask;
use tg_service_render::{BorderCharacter, BorderStyle, CustomBorder};
use tg_service_rich_text::TextMode;
use tg_service_text_layout::{DrawTextParams, TextAlign, TextWrapMode};

mod align;
mod base;
#[path = "libraries/char.rs"]
mod chars;
mod color;
mod date;
mod debug;
mod draw;
mod encoding;
mod events;
mod file;
mod game;
mod i18n;
mod image;
mod ime;
mod keyboard;
mod loader;
mod math;
mod measurement;
mod random;
mod serialization;
mod slice;
mod string;
mod table;
mod timer;
mod utf8;

use measurement::{
  draw_target_size, draw_text_parameters, parse_color, parse_draw_target, parse_draw_text_params,
  positional_table, positive_u16,
};
use string::rich_text_params;

const MAX_HOST_COMMANDS_PER_CALLBACK: usize = 4096;

/// Build and register the Lua libraries API in the supplied VM and host context.
///
/// # Arguments
///
/// * `lua` - The Lua VM in which values and callbacks are created.
/// * `environment` - The environment.
/// * `state` - The state.
///
/// # Errors
///
/// Propagate Lua allocation, table construction, or function registration errors while installing
/// this library.
pub fn install(lua: &Lua, environment: &Table, state: SharedApiState) -> mlua::Result<()> {
  let base = base::base(lua)?;
  environment.set("base", base.clone())?;
  for name in [
    "ipairs",
    "pairs",
    "next",
    "select",
    "rawequal",
    "rawget",
    "rawset",
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
  environment.set("date", date::date(lua)?)?;
  environment.set("char", chars::char_lib(lua)?)?;
  environment.set("align", align::align(lua, state.clone())?)?;
  environment.set("measurement", measurement::measurement(lua, state.clone())?)?;
  environment.set("random", random::random(lua, state.clone())?)?;
  environment.set("slice", slice::slice(lua, state.clone())?)?;
  environment.set("timer", timer::timer(lua, state.clone())?)?;
  environment.set("serialization", serialization::serialization(lua)?)?;
  environment.set("encoding", encoding::encoding(lua)?)?;
  environment.set("draw", draw::draw(lua, state.clone())?)?;
  environment.set("debug", debug::debug(lua, state.clone())?)?;
  environment.set("game", game::game(lua, state.clone())?)?;
  environment.set("i18n", i18n::i18n(lua, state.clone())?)?;
  environment.set("image", image::image(lua, state.clone())?)?;
  environment.set("keyboard", keyboard::keyboard(lua, state.clone())?)?;
  environment.set("ime", ime::ime(lua, state.clone())?)?;
  environment.set("events", events::events(lua, state.clone())?)?;
  environment.set("loader", loader::loader(lua, environment, state.clone())?)?;
  environment.set("file", file::file(lua, state)?)?;
  Ok(())
}

/// Resolve an asynchronous asset path, returning no path when the resource is unavailable.
///
/// # Arguments
///
/// * `root` - The deployment asset root.
/// * `relative` - The validated path within that root.
/// * `kind` - The required file or directory kind.
/// * `method` - The Lua method used in argument errors.
///
/// # Errors
///
/// Return a Lua argument error for an unsafe path or a sandbox escape.
fn resolve_request_path(
  root: &Path,
  relative: &crate::path::SafeRelativePath,
  kind: crate::path::SandboxPathKind,
  method: &str,
) -> mlua::Result<Option<PathBuf>> {
  use crate::path::SandboxPathError;
  match crate::path::resolve_sandbox_path(root, relative, kind) {
    Ok(path) => Ok(Some(path)),
    Err(
      SandboxPathError::RootUnavailable
      | SandboxPathError::NotFound
      | SandboxPathError::ParentUnavailable
      | SandboxPathError::NotFile
      | SandboxPathError::NotDirectory,
    ) => Ok(None),
    Err(error) => Err(args::message(method, format!("unsafe asset path: {error}"))),
  }
}

fn function_value(function: Function) -> Value {
  Value::Function(function)
}

/// Reject calls that require a game session.
///
/// # Errors
///
/// Return a Lua error naming the method when the current session is a screensaver.
fn require_game(state: &super::LuaApiState, method: &str) -> mlua::Result<()> {
  if state.context.session_kind != LuaSessionKind::Game {
    return Err(args::message(method, "method requires a game session"));
  }
  Ok(())
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

fn with_pool<R>(
  state: &SharedApiState,
  method: &str,
  operation: impl FnOnce(&crate::LuaObjectPool) -> mlua::Result<R>,
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
  operation: impl FnOnce(&mut crate::LuaObjectPool) -> mlua::Result<R>,
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
