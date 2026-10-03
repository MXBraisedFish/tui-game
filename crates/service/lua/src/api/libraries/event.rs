//! Lua event library bindings with validated arguments and session-owned host access.

use super::*;

/// Build and register the Lua event API in the supplied VM and host context.
///
/// # Errors
///
/// Propagate Lua allocation, table construction, or function registration errors while installing
/// this library.
pub(super) fn event(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, command) in [
    ("skip_action", LuaHostCommand::SkipActions),
    ("clear_action", LuaHostCommand::ClearActions),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method = if matches!(command, LuaHostCommand::SkipActions) {
          "event.skip_action"
        } else {
          "event.clear_action"
        };
        let mut api = state.borrow_mut();
        if api.context.session_kind != LuaSessionKind::Game {
          ignore_once(&mut api, method, "method requires a game session");
          return Ok(());
        }
        drop(api);
        args::no_args(method, values)?;
        push_host_command(&mut state.borrow_mut(), command.clone());
        Ok(())
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}
