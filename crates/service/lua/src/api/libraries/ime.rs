//! Independent session keyboard event subscriptions.

use super::*;

/// Register keyboard subscriptions without changing host or widget input ownership.
///
/// # Errors
///
/// Return a Lua error when allocating the library or validating arguments.
pub(super) fn ime(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, actions, enabled) in [
    ("receive_action_event", true, true),
    ("reject_action_event", true, false),
    ("receive_key_event", false, true),
    ("reject_key_event", false, false),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method = match name {
          "receive_action_event" => "ime.receive_action_event",
          "reject_action_event" => "ime.reject_action_event",
          "receive_key_event" => "ime.receive_key_event",
          _ => "ime.reject_key_event",
        };
        require_game(&state.borrow(), method)?;
        args::no_args(method, values)?;
        let mut api = state.borrow_mut();
        let current = if actions {
          api.input.actions
        } else {
          api.input.keys
        };
        if current == enabled {
          return Ok(true);
        }
        if actions {
          api.input.actions = enabled;
        } else {
          api.input.keys = enabled;
        }
        if !enabled {
          api.input.close(actions, !actions);
          push_host_command(
            &mut api,
            LuaHostCommand::InputRejected {
              actions,
              keys: !actions,
            },
          );
        }
        Ok(true)
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}
