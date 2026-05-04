//! get_key 表构造

use mlua::{Lua, Table};
use serde_json::Value as JsonValue;

use super::{key_display, lua_table_value};
use crate::host_engine::boot::preload::game_modules::GameModule;

/// 构造 action_value 表。
pub fn build_key_table(
    lua: &Lua,
    game_module: Option<&GameModule>,
    keybinds: &JsonValue,
    ui_actions: &JsonValue,
    requested_action: Option<&str>,
) -> mlua::Result<Table> {
    if let Some(game_module) = game_module {
        return build_game_key_table(lua, game_module, keybinds, requested_action);
    }

    build_ui_key_table(lua, ui_actions, requested_action)
}

fn build_game_key_table(
    lua: &Lua,
    game_module: &GameModule,
    keybinds: &JsonValue,
    requested_action: Option<&str>,
) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    match requested_action {
        Some(action) => {
            if let Some(action_table) =
                build_single_game_action_table(lua, game_module, keybinds, action)?
            {
                return Ok(action_table);
            }
        }
        None => {
            for action in game_module.game.actions.keys() {
                if let Some(action_table) =
                    build_single_game_action_table(lua, game_module, keybinds, action)?
                {
                    table.set(action.as_str(), action_table)?;
                }
            }
        }
    }

    Ok(table)
}

fn build_single_game_action_table(
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
    table.set(
        "key_display",
        build_key_display_table(
            lua,
            &action_binding.key,
            user_key,
            game_module.game.case_sensitive,
        )?,
    )?;

    Ok(Some(table))
}

fn build_ui_key_table(
    lua: &Lua,
    ui_actions: &JsonValue,
    requested_action: Option<&str>,
) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let Some(actions) = ui_actions.as_object() else {
        return Ok(table);
    };

    match requested_action {
        Some(action) => {
            if let Some(action_table) = build_single_ui_action_table(lua, actions.get(action))? {
                return Ok(action_table);
            }
        }
        None => {
            for (action, action_value) in actions {
                if let Some(action_table) = build_single_ui_action_table(lua, Some(action_value))? {
                    table.set(action.as_str(), action_table)?;
                }
            }
        }
    }

    Ok(table)
}

fn build_single_ui_action_table(
    lua: &Lua,
    action_value: Option<&JsonValue>,
) -> mlua::Result<Option<Table>> {
    let Some(action_value) = action_value else {
        return Ok(None);
    };
    let Some(key) = action_value.get("key") else {
        return Ok(None);
    };

    let table = lua.create_table()?;
    table.set("key", lua_table_value::json_to_lua_value(lua, key)?)?;
    table.set(
        "key_name",
        action_value
            .get("name")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
    )?;
    table.set("key_user", lua_table_value::json_to_lua_value(lua, key)?)?;
    table.set("key_display", build_key_display_table(lua, key, key, false)?)?;
    Ok(Some(table))
}

fn build_key_display_table(
    lua: &Lua,
    key: &JsonValue,
    user_key: &JsonValue,
    case_sensitive: bool,
) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set(
        "key",
        lua_table_value::json_to_lua_value(
            lua,
            &key_display::display_key_value(key, case_sensitive),
        )?,
    )?;
    table.set(
        "key_user",
        lua_table_value::json_to_lua_value(
            lua,
            &key_display::display_key_value(user_key, case_sensitive),
        )?,
    )?;
    Ok(table)
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
