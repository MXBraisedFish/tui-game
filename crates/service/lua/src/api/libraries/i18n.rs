use super::*;
use crate::LuaI18nEventKind;

pub(super) fn i18n(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;

  let create_state = state.clone();
  source.raw_set(
    "create",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "i18n.create";
      let parameters = args::positional(
        lua,
        method,
        values,
        &[],
        &["language_code", "callback_language_code"],
      )?;
      let table = parameters.options();
      let system_language = create_state.borrow().context.language_code.clone();
      let language_code = args::optional_string(
        table,
        method,
        "language_code",
        Some(system_language.as_str()),
      )?
      .expect("language_code has a default");
      let callback_language_code =
        args::optional_string(table, method, "callback_language_code", Some("en_us"))?
          .expect("callback_language_code has a default");
      validate_language_code(method, "language_code", &language_code)?;
      validate_language_code(method, "callback_language_code", &callback_language_code)?;

      let mut api = create_state.borrow_mut();
      if api.i18n.created || api.i18n.loading {
        return Ok(Value::Nil);
      }
      api.i18n.created = true;
      api.i18n.loading = true;
      let request_id = api.next_i18n_request_id;
      api.next_i18n_request_id = api.next_i18n_request_id.wrapping_add(1).max(1);
      let assets_root = api.context.assets_root.clone();
      push_host_command(
        &mut api,
        LuaHostCommand::I18nRequest {
          request_id,
          task: FileTask::LuaLoadI18n {
            assets_root,
            language_code: language_code.clone(),
            callback_language_code: callback_language_code.clone(),
          },
          kind: LuaI18nEventKind::Created,
          language_code,
          callback_language_code,
        },
      );
      Ok(Value::Integer(request_id as i64))
    })?,
  )?;

  let value_state = state.clone();
  source.raw_set(
    "get_value",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "i18n.get_value";
      let parameters = args::positional(lua, method, values, &["namespace", "key"], &[])?;
      let namespace = args::string(
        parameters.required(0, method, "namespace")?,
        method,
        "namespace",
      )?;
      let key = args::string(parameters.required(1, method, "key")?, method, "key")?;
      validate_lookup_name(method, "namespace", &namespace)?;
      validate_lookup_name(method, "key", &key)?;
      let api = value_state.borrow();
      if let Some(value) = api
        .i18n
        .namespaces
        .get(&namespace)
        .and_then(|values| values.get(&key))
      {
        return Ok(value.clone());
      }
      let missing_key = if key.starts_with(&format!("{namespace}.")) {
        key
      } else {
        format!("{namespace}.{key}")
      };
      Ok(
        api
          .context
          .missing_i18n_template
          .replace("{value:missing_key}", &missing_key),
      )
    })?,
  )?;

  let language_state = state.clone();
  source.raw_set(
    "get_language_code",
    lua.create_function(move |_, values: MultiValue| {
      args::no_args("i18n.get_language_code", values)?;
      Ok(language_state.borrow().context.language_code.clone())
    })?,
  )?;

  let reload_state = state;
  source.raw_set(
    "reload",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "i18n.reload";
      let parameters = args::positional(
        lua,
        method,
        values,
        &[],
        &["language_code", "callback_language_code"],
      )?;
      let table = parameters.options();
      let system_language = reload_state.borrow().context.language_code.clone();
      let language_code = args::optional_string(
        table,
        method,
        "language_code",
        Some(system_language.as_str()),
      )?
      .expect("language_code has a default");
      let callback_language_code =
        args::optional_string(table, method, "callback_language_code", Some("en_us"))?
          .expect("callback_language_code has a default");
      validate_language_code(method, "language_code", &language_code)?;
      validate_language_code(method, "callback_language_code", &callback_language_code)?;

      let mut api = reload_state.borrow_mut();
      if !api.i18n.created {
        return Err(args::message(method, "i18n instance has not been created"));
      }
      if api.i18n.loading {
        return Ok(Value::Nil);
      }
      api.i18n.loading = true;
      let request_id = api.next_i18n_request_id;
      api.next_i18n_request_id = api.next_i18n_request_id.wrapping_add(1).max(1);
      let assets_root = api.context.assets_root.clone();
      push_host_command(
        &mut api,
        LuaHostCommand::I18nRequest {
          request_id,
          task: FileTask::LuaLoadI18n {
            assets_root,
            language_code: language_code.clone(),
            callback_language_code: callback_language_code.clone(),
          },
          kind: LuaI18nEventKind::Reloaded,
          language_code,
          callback_language_code,
        },
      );
      Ok(Value::Integer(request_id as i64))
    })?,
  )?;

  readonly::proxy(lua, source)
}

fn validate_language_code(method: &str, name: &str, value: &str) -> mlua::Result<()> {
  if value.is_empty()
    || value.len() > 64
    || !value
      .bytes()
      .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
  {
    return Err(args::message(
      method,
      format!("invalid parameter '{name}': expected a portable language code"),
    ));
  }
  Ok(())
}

fn validate_lookup_name(method: &str, name: &str, value: &str) -> mlua::Result<()> {
  if value.is_empty()
    || value.len() > 512
    || value
      .chars()
      .any(|character| matches!(character, '\0' | '\r' | '\n'))
  {
    return Err(args::message(
      method,
      format!("invalid parameter '{name}': expected a non-empty i18n name"),
    ));
  }
  Ok(())
}
