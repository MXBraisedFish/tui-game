//! Independent session keyboard event subscriptions.

use super::*;

/// Register keyboard subscriptions without changing host or widget input ownership.
///
/// # Errors
///
/// Return a Lua error when allocating the library or validating arguments.
pub(super) fn keyboard(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, actions, enabled) in [
    ("enable_action_event", true, true),
    ("disable_action_event", true, false),
    ("enable_key_event", false, true),
    ("disable_key_event", false, false),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method = match name {
          "enable_action_event" => "keyboard.enable_action_event",
          "disable_action_event" => "keyboard.disable_action_event",
          "enable_key_event" => "keyboard.enable_key_event",
          _ => "keyboard.disable_key_event",
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
        if !enabled && api.commands.len() >= MAX_HOST_COMMANDS_PER_CALLBACK {
          return Ok(false);
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
