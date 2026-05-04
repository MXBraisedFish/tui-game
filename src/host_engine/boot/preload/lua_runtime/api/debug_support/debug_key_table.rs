//! get_key 表构造

use mlua::{Lua, Table};
use serde_json::Value as JsonValue;

use super::lua_table_value;
use crate::host_engine::boot::preload::game_modules::GameModule;

/// 构造 action_value 表。
pub fn build_key_table(
    lua: &Lua,
    game_module: Option<&GameModule>,
    keybinds: &JsonValue,
    requested_action: Option<&str>,
) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let Some(game_module) = game_module else {
        return Ok(table);
    };

    match requested_action {
        Some(action) => {
            if let Some(action_table) =
                build_single_action_table(lua, game_module, keybinds, action)?
            {
                table.set(action, action_table)?;
            }
        }
        None => {
            for action in game_module.game.actions.keys() {
                if let Some(action_table) =
                    build_single_action_table(lua, game_module, keybinds, action)?
                {
                    table.set(action.as_str(), action_table)?;
                }
            }
        }
    }

    Ok(table)
}

fn build_single_action_table(
    lua: &Lua,
    game_module: &GameModule,
    keybinds: &JsonValue,
    action: &str,
) -> mlua::Result<Option<Table>> {
    let Some(action_binding) = game_module.game.actions.get(action) else {
        return Ok(None);
    };

    let table = lua.create_table()?;
    let user_key =
        find_user_key(keybinds, game_module.uid.as_str(), action).unwrap_or(&action_binding.key);

    table.set(
        "key",
        lua_table_value::json_to_lua_value(lua, &action_binding.key)?,
    )?;
    table.set("key_name", action_binding.key_name.as_str())?;
    table.set(
        "key_user",
        lua_table_value::json_to_lua_value(lua, user_key)?,
    )?;

    Ok(Some(table))
}

fn find_user_key<'a>(
    keybinds: &'a JsonValue,
    game_uid: &str,
    action: &str,
) -> Option<&'a JsonValue> {
    keybinds
        .get(game_uid)
        .and_then(|game_keybinds| game_keybinds.get(action))
        .and_then(|action_keybind| action_keybind.get("key_user"))
}
