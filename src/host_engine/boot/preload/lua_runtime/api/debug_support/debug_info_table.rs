//! get_game_info 表构造

use mlua::{Lua, Table, Value};

use super::lua_table_value;
use crate::host_engine::boot::preload::game_modules::GameModule;

/// 构造 game_info 表。
pub fn build_game_info_table(lua: &Lua, game_module: &GameModule) -> mlua::Result<Table> {
    let table = lua.create_table()?;

    table.set("uid", game_module.uid.as_str())?;
    table.set("package", game_module.package.package.as_str())?;
    table.set("mod_name", game_module.package.mod_name.as_str())?;
    table.set("introduction", game_module.package.introduction.as_str())?;
    table.set("author", game_module.package.author.as_str())?;
    table.set("game_name", game_module.package.game_name.as_str())?;
    table.set("description", game_module.package.description.as_str())?;
    table.set("detail", game_module.package.detail.as_str())?;
    table.set(
        "icon",
        lua_table_value::json_to_lua_value(lua, &game_module.package.icon)?,
    )?;
    table.set(
        "banner",
        lua_table_value::json_to_lua_value(lua, &game_module.package.banner)?,
    )?;
    table.set(
        "api",
        lua_table_value::json_to_lua_value(lua, &game_module.game.api)?,
    )?;
    table.set("entry", game_module.game.entry.as_str())?;
    table.set("save", game_module.game.save)?;
    table.set(
        "best_none",
        optional_string_to_lua(lua, game_module.game.best_none.as_deref())?,
    )?;
    table.set("min_width", game_module.game.min_width)?;
    table.set("min_height", game_module.game.min_height)?;
    table.set("write", game_module.game.write)?;
    table.set("case_sensitive", game_module.game.case_sensitive)?;
    table.set("actions", build_actions_table(lua, game_module)?)?;
    table.set("runtime", build_runtime_table(lua, game_module)?)?;

    Ok(table)
}

fn build_actions_table(lua: &Lua, game_module: &GameModule) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (action, action_binding) in &game_module.game.actions {
        let action_table = lua.create_table()?;
        action_table.set(
            "key",
            lua_table_value::json_to_lua_value(lua, &action_binding.key)?,
        )?;
        action_table.set("key_name", action_binding.key_name.as_str())?;
        table.set(action.as_str(), action_table)?;
    }
    Ok(table)
}

fn build_runtime_table(lua: &Lua, game_module: &GameModule) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("target_fps", game_module.game.runtime.target_fps)?;
    Ok(table)
}

fn optional_string_to_lua(lua: &Lua, value: Option<&str>) -> mlua::Result<Value> {
    match value {
        Some(value) => Ok(Value::String(lua.create_string(value)?)),
        None => Ok(Value::Nil),
    }
}
