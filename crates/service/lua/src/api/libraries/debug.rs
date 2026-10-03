//! Lua debug library bindings with validated arguments and session-owned host access.

use super::*;

/// Build and register the Lua debug API in the supplied VM and host context.
///
/// # Errors
///
/// Propagate Lua allocation, table construction, or function registration errors while installing
/// this library.
pub(super) fn debug(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  source.raw_set("VERSION", "Lua 5.4 / TUI GAME API 1")?;
  for (name, value) in [
    ("TRACE", "trace"),
    ("DEBUG", "debug"),
    ("INFO", "info"),
    ("WARN", "warn"),
    ("ERROR", "error"),
    ("FATAL", "fatal"),
  ] {
    source.raw_set(name, value)?;
  }
  for (name, level) in [
    ("print", None),
    ("info", Some("info")),
    ("warn", Some("warn")),
    ("error", Some("error")),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let method = match name {
          "print" => "debug.print",
          "info" => "debug.info",
          "warn" => "debug.warn",
          _ => "debug.error",
        };
        if !state.borrow().context.debug_enabled {
          ignore_once(&mut state.borrow_mut(), method, "debug mode is disabled");
          return Ok(());
        }
        if name == "print" {
          let parameters = args::positional(
            lua,
            method,
            values,
            &["message"],
            &["title", "level", "time", "type_head"],
          )?;
          let table = parameters.options();
          let message = args::dynamic_text(
            parameters.required(0, method, "message")?,
            method,
            "message",
          )?;
          let title = args::optional_dynamic_text(table, method, "title", None)?;
          let time = args::optional_bool(table, method, "time", false)?;
          let type_head = args::optional_bool(table, method, "type_head", false)?;
          let level = args::optional_string(table, method, "level", None)?;
          let level = level
            .map(|level| level.to_ascii_lowercase())
            .map(|level| match level.as_str() {
              "trace" | "debug" | "info" | "warn" | "error" | "fatal" => Ok(level),
              _ => Err(args::message(
                method,
                "level must be trace, debug, info, warn, error, fatal, or nil",
              )),
            })
            .transpose()?;
          enqueue_debug_print(
            &mut state.borrow_mut(),
            message,
            title,
            time,
            level,
            type_head,
          );
        } else {
          let parameters = args::positional(lua, method, values, &["message"], &[])?;
          let message = args::dynamic_text(
            parameters.required(0, method, "message")?,
            method,
            "message",
          )?;
          enqueue_debug_print(
            &mut state.borrow_mut(),
            message,
            None,
            true,
            level.map(str::to_string),
            true,
          );
        }
        Ok(())
      })?,
    )?;
  }
  source.raw_set(
    "assert",
    lua.create_function(|lua, values: MultiValue| {
      let parameters = args::positional(lua, "debug.assert", values, &["value"], &["message"])?;
      // Explicit nil can be asserted; an omitted required position is still an argument error.

      let value = parameters.get(0);
      if matches!(value, Value::Nil | Value::Boolean(false)) {
        let message = args::optional_dynamic_text(
          parameters.options(),
          "debug.assert",
          "message",
          Some("assertion failed"),
        )?
        .unwrap();
        Err(mlua::Error::RuntimeError(message))
      } else {
        Ok(value)
      }
    })?,
  )?;
  source.raw_set("pcall", protected(lua, false, state.clone())?)?;
  source.raw_set("xpcall", protected(lua, true, state)?)?;
  readonly::proxy(lua, source)
}

fn protected(lua: &Lua, extended: bool, state: SharedApiState) -> mlua::Result<Function> {
  lua.create_function(move |lua, values: MultiValue| {
    let method = if extended {
      "debug.xpcall"
    } else {
      "debug.pcall"
    };
    let required = if extended {
      &["func", "error_callback"][..]
    } else {
      &["func"][..]
    };
    let values = args::variadic(method, values, required)?;
    let value = values[0].clone();
    let Value::Function(function) = value else {
      return Err(args::invalid(method, "func", "function", &value));
    };
    let error_callback = if extended {
      let value = values[1].clone();
      let Value::Function(function) = value else {
        return Err(args::invalid(method, "error_callback", "function", &value));
      };
      Some(function)
    } else {
      None
    };
    let call_values = values.into_iter().skip(required.len()).collect::<Vec<_>>();
    match function.call::<MultiValue>(MultiValue::from_vec(call_values)) {
      Ok(result) => {
        if state.borrow().fatal_budget_exceeded || state.borrow().fatal_api_error {
          return Err(mlua::Error::RuntimeError(
            "fatal Lua API resource limit exceeded".to_string(),
          ));
        }
        let mut output = Vec::with_capacity(result.len() + 1);
        output.push(Value::Boolean(true));
        output.extend(result);
        Ok(MultiValue::from_vec(output))
      }
      Err(error) => {
        if state.borrow().fatal_budget_exceeded
          || state.borrow().fatal_api_error
          || is_memory_error(&error)
        {
          return Err(error);
        }
        let mut error_value = Value::String(lua.create_string(error.to_string())?);
        if let Some(handler) = error_callback {
          error_value = handler.call(error_value)?;
        }
        Ok(MultiValue::from_vec(vec![
          Value::Boolean(false),
          error_value,
        ]))
      }
    }
  })
}

fn is_memory_error(error: &mlua::Error) -> bool {
  match error {
    mlua::Error::MemoryError(_) => true,
    mlua::Error::BadArgument { cause, .. } | mlua::Error::CallbackError { cause, .. } => {
      is_memory_error(cause)
    }
    _ => false,
  }
}
