//! Input-method controls, committed terminal text, and text clipboard writes.

use super::*;

/// Register input-method controls and session-local text subscriptions.
///
/// # Errors
///
/// Return a Lua error for invalid arguments, a restricted session, or allocation failure.
pub(super) fn ime(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  let clipboard_state = state.clone();
  source.raw_set(
    "write_clipboard",
    lua.create_function(move |_, values: MultiValue| {
      let method = "ime.write_clipboard";
      require_game(&clipboard_state.borrow(), method)?;
      let text = args::string(args::one(method, "text", values)?, method, "text")?;
      if text.contains('\0') {
        return Err(args::message(method, "parameter 'text' cannot contain NUL"));
      }
      let service = clipboard_state.borrow().context.clipboard.clone();
      Ok(service.is_some_and(|service| service.borrow_mut().write_text(&text)))
    })?,
  )?;
  let lock_state = state.clone();
  source.raw_set(
    "lock",
    lua.create_function(move |_, values: MultiValue| {
      let method = "ime.lock";
      require_game(&lock_state.borrow(), method)?;
      args::no_args(method, values)?;
      let mut api = lock_state.borrow_mut();
      let Some(service) = api.context.input_method.clone() else {
        return Ok(false);
      };
      let mut service = service.borrow_mut();
      let success = service.restrict_input_method() && service.reconcile_now();
      if success {
        api.input.ime_locked = true;
      }
      Ok(success)
    })?,
  )?;
  let unlock_state = state.clone();
  source.raw_set(
    "unlock",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "ime.unlock";
      require_game(&unlock_state.borrow(), method)?;
      let parsed = args::positional(lua, method, values, &[], &["restore"])?;
      let restore = args::optional_bool(parsed.options(), method, "restore", true)?;
      let mut api = unlock_state.borrow_mut();
      let Some(service) = api.context.input_method.clone() else {
        return Ok(false);
      };
      let success = service.borrow_mut().unlock_input_method(restore);
      if success {
        api.input.ime_locked = false;
      }
      Ok(success)
    })?,
  )?;
  for (name, enabled) in [("receive_input_event", true), ("reject_input_event", false)] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method = format!("ime.{name}");
        require_game(&state.borrow(), &method)?;
        args::no_args(&method, values)?;
        let mut api = state.borrow_mut();
        if api.input.text != enabled {
          api.input.text = enabled;
          if !enabled {
            api.input.close_text();
          }
        }
        Ok(true)
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}
