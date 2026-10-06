//! Game-session lifecycle, frame callbacks, host commands, and optional save support.

use super::*;

/// Build and register the Lua game API in the supplied VM and host context.
///
/// # Errors
///
/// Propagate Lua allocation, table construction, or function registration errors while installing
/// this library.
pub(super) fn game(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, command) in [
    ("exit_game", LuaHostCommand::ExitGame),
    ("save_game", LuaHostCommand::SaveGame),
    ("save_best", LuaHostCommand::SaveBest),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method = match command {
          LuaHostCommand::ExitGame => "game.exit_game",
          LuaHostCommand::SaveGame => "game.save_game",
          _ => "game.save_best",
        };
        require_game(&state.borrow(), method)?;
        args::no_args(method, values)?;
        let mut api = state.borrow_mut();
        let invalid_state = match command {
          LuaHostCommand::ExitGame => match api.phase {
            LuaCallPhase::Init => Some("Init"),
            LuaCallPhase::SaveGame => Some("SaveGame"),
            LuaCallPhase::SaveBest => Some("SaveBest"),
            _ => None,
          },
          LuaHostCommand::SaveGame if api.phase == LuaCallPhase::SaveGame => Some("SaveGame"),
          LuaHostCommand::SaveBest if api.phase == LuaCallPhase::SaveBest => Some("SaveBest"),
          _ => None,
        };
        if let Some(callback) = invalid_state {
          return Err(args::message(
            method,
            format!("invalid_state: {method} cannot be called during {callback}"),
          ));
        }
        push_host_command(&mut api, command.clone());
        Ok(())
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}
