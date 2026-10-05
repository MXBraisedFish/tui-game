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
        require_game(&state.borrow(), method)?;
        args::no_args(method, values)?;
        push_host_command(&mut state.borrow_mut(), command.clone());
        Ok(())
      })?,
    )?;
  }
  for (name, enabled) in [
    ("enable_focus_release", true),
    ("disable_focus_release", false),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method = if enabled {
          "event.enable_focus_release"
        } else {
          "event.disable_focus_release"
        };
        require_game(&state.borrow(), method)?;
        args::no_args(method, values)?;
        state.borrow_mut().input.focus_release = enabled;
        Ok(true)
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}
