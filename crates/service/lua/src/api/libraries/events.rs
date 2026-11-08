//! Lua events library bindings with validated arguments and session-owned host access.

use super::*;

/// Build and register the Lua events API in the supplied VM and host context.
///
/// # Errors
///
/// Propagate Lua allocation, table construction, or function registration errors while installing
/// this library.
pub(super) fn events(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, command) in [
    ("skip_actions", LuaHostCommand::SkipActions),
    ("clear_actions", LuaHostCommand::ClearActions),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method = if matches!(command, LuaHostCommand::SkipActions) {
          "events.skip_actions"
        } else {
          "events.clear_actions"
        };
        require_game(&state.borrow(), method)?;
        if !state.borrow().context.debug_enabled {
          ignore_once(&mut state.borrow_mut(), method, "debug mode is disabled");
          return Ok(());
        }
        args::no_args(method, values)?;
        push_host_command(&mut state.borrow_mut(), command.clone());
        Ok(())
      })?,
    )?;
  }
  for (name, enabled) in [
    ("enable_focus_release_events", true),
    ("disable_focus_release_events", false),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method = if enabled {
          "events.enable_focus_release_events"
        } else {
          "events.disable_focus_release_events"
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
