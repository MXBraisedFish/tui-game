use super::*;

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
      lua.create_function(move |_, values: MultiValue| {
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
          let table = args::named(
            method,
            values,
            &["message", "title", "level", "time", "type_head"],
          )?;
          let message = args::dynamic_text(
            args::required(&table, method, "message")?,
            method,
            "message",
          )?;
          let title = args::optional_dynamic_text(&table, method, "title", None)?;
          let time = args::optional_bool(&table, method, "time", false)?;
          let type_head = args::optional_bool(&table, method, "type_head", false)?;
          let level = args::optional_string(&table, method, "level", None)?;
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
          let message =
            args::dynamic_text(args::one(method, "message", values)?, method, "message")?;
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
    lua.create_function(|_, values: MultiValue| {
      let table = args::named("debug.assert", values, &["value", "message"])?;
      // Lua 的 nil 无法作为表字段保留下来，因此省略 value 与显式传入 nil
      // 都应进入断言失败分支，而不是被通用必填参数校验拦截。
      let value = table.get::<Value>("value")?;
      if matches!(value, Value::Nil | Value::Boolean(false)) {
        let message =
          args::optional_dynamic_text(&table, "debug.assert", "message", Some("assertion failed"))?
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
    let allowed = if extended {
      &["func", "error_callback", "values"][..]
    } else {
      &["func", "values"][..]
    };
    let table = args::named(method, values, allowed)?;
    let value = args::required(&table, method, "func")?;
    let Value::Function(function) = value else {
      return Err(args::invalid(method, "func", "function", &value));
    };
    let call_values = if matches!(table.get::<Value>("values")?, Value::Nil) {
      Vec::new()
    } else {
      args::values(&table, method)?
    };
    match function.call::<MultiValue>(MultiValue::from_vec(call_values)) {
      Ok(result) => {
        if state.borrow().fatal_budget_exceeded || state.borrow().fatal_api_error {
          return Err(mlua::Error::RuntimeError(
            "fatal Lua API resource limit exceeded".to_string(),
          ));
        }
        let output = lua.create_table()?;
        output.raw_set("ok", true)?;
        let packed = lua.create_table()?;
        let count = result.len();
        for (index, value) in result.into_iter().enumerate() {
          packed.raw_set(index + 1, value)?;
        }
        packed.raw_set("n", count)?;
        output.raw_set("values", packed)?;
        Ok(output)
      }
      Err(error) => {
        if state.borrow().fatal_budget_exceeded
          || state.borrow().fatal_api_error
          || is_memory_error(&error)
        {
          return Err(error);
        }
        let mut error_value = Value::String(lua.create_string(error.to_string())?);
        if extended {
          if let Value::Function(handler) = table.get::<Value>("error_callback")? {
            error_value = handler.call(error_value)?;
          }
        }
        let output = lua.create_table()?;
        output.raw_set("ok", false)?;
        output.raw_set("error", error_value)?;
        Ok(output)
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
