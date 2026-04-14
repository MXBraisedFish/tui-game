use anyhow::{Result, anyhow};
use mlua::{Function, Lua, RegistryKey, Table, Value};
use serde_json::{Map, Number, Value as JsonValue};

use crate::app::i18n;
use crate::core::event::InputEvent;
use crate::core::runtime::LaunchMode;
use crate::core::save as runtime_save;
use crate::core::stats as runtime_stats;
use crate::game::registry::GameDescriptor;
use crate::lua::engine::anyhow_lua_error;

pub(crate) fn validate_required_callbacks(lua: &Lua, game: &GameDescriptor) -> Result<()> {
    require_callback(lua, "init_game", "init_game(state)")?;
    require_callback(lua, "handle_event", "handle_event(state, event)")?;
    require_callback(lua, "render", "render(state)")?;
    require_callback(lua, "exit_game", "exit_game(state)")?;
    if game.has_best_score {
        require_callback(lua, "save_best_score", "save_best_score(state)")?;
    }
    if game.save {
        require_callback(lua, "save_game", "save_game(state)")?;
    }
    Ok(())
}

pub(crate) fn initialize_state(
    lua: &Lua,
    game: &GameDescriptor,
    launch_mode: LaunchMode,
) -> Result<RegistryKey> {
    let init_game: Function = lua
        .globals()
        .get("init_game")
        .map_err(|err| missing_entry_callback_error("init_game(state)", &err.to_string()))?;
    let incoming_state = match launch_mode {
        LaunchMode::Continue => runtime_save::load_continue(&game.id)?
            .map(|json| json_to_lua_value(lua, &json))
            .transpose()?
            .unwrap_or(Value::Nil),
        LaunchMode::New => Value::Nil,
    };
    let state = init_game
        .call::<Value>(incoming_state)
        .map_err(anyhow_lua_error)?;
    lua.create_registry_value(state).map_err(anyhow_lua_error)
}

pub(crate) fn call_handle_event(
    lua: &Lua,
    state_key: &RegistryKey,
    event: &InputEvent,
) -> Result<RegistryKey> {
    let handle_event: Function = lua
        .globals()
        .get("handle_event")
        .map_err(|err| missing_entry_callback_error("handle_event(state, event)", &err.to_string()))?;
    let state = lua
        .registry_value::<Value>(state_key)
        .map_err(anyhow_lua_error)?;
    let event_table = to_lua_event_table(lua, event).map_err(anyhow_lua_error)?;
    let new_state = handle_event
        .call::<Value>((state, event_table))
        .map_err(anyhow_lua_error)?;
    lua.create_registry_value(new_state).map_err(anyhow_lua_error)
}

pub(crate) fn call_render(lua: &Lua, state_key: &RegistryKey) -> Result<()> {
    let render: Function = lua
        .globals()
        .get("render")
        .map_err(|err| missing_entry_callback_error("render(state)", &err.to_string()))?;
    let state = lua
        .registry_value::<Value>(state_key)
        .map_err(anyhow_lua_error)?;
    render.call::<()>(state).map_err(anyhow_lua_error)?;
    Ok(())
}

pub(crate) fn call_exit_game(lua: &Lua, state_key: &RegistryKey) -> Result<RegistryKey> {
    let exit_game: Function = lua
        .globals()
        .get("exit_game")
        .map_err(|err| missing_entry_callback_error("exit_game(state)", &err.to_string()))?;
    let state = lua
        .registry_value::<Value>(state_key)
        .map_err(anyhow_lua_error)?;
    let new_state = exit_game.call::<Value>(state).map_err(anyhow_lua_error)?;
    lua.create_registry_value(new_state).map_err(anyhow_lua_error)
}

pub(crate) fn persist_best_score(
    lua: &Lua,
    state_key: &RegistryKey,
    game: &GameDescriptor,
) -> Result<()> {
    if !game.has_best_score {
        return Ok(());
    }
    let save_best_score: Function = lua
        .globals()
        .get("save_best_score")
        .map_err(|err| missing_entry_callback_error("save_best_score(state)", &err.to_string()))?;
    let state = lua
        .registry_value::<Value>(state_key)
        .map_err(anyhow_lua_error)?;
    let value = save_best_score
        .call::<Value>(state)
        .map_err(anyhow_lua_error)?;
    if matches!(value, Value::Nil) {
        return Ok(());
    }
    runtime_stats::write_runtime_best_score(game.id.as_str(), &lua_value_to_json(&value)?)?;
    Ok(())
}

pub(crate) fn persist_save_game(
    lua: &Lua,
    state_key: &RegistryKey,
    game: &GameDescriptor,
) -> Result<()> {
    if !game.save {
        return Ok(());
    }
    let save_game: Function = lua
        .globals()
        .get("save_game")
        .map_err(|err| missing_entry_callback_error("save_game(state)", &err.to_string()))?;
    let state = lua
        .registry_value::<Value>(state_key)
        .map_err(anyhow_lua_error)?;
    let value = save_game.call::<Value>(state).map_err(anyhow_lua_error)?;
    if matches!(value, Value::Nil) {
        return Ok(());
    }
    runtime_save::save_continue(game.id.as_str(), &lua_value_to_json(&value)?)?;
    Ok(())
}

fn require_callback(lua: &Lua, global_name: &str, label: &str) -> Result<()> {
    let _: Function = lua
        .globals()
        .get(global_name)
        .map_err(|err| {
            anyhow!(
                "{}",
                i18n::t_or(
                    "host.error.missing_required_callback_api",
                    "Missing required Callback API: {err}",
                )
                .replace("{err}", &format!("{label}: {err}"))
            )
        })?;
    Ok(())
}

fn missing_entry_callback_error(label: &str, err: &str) -> anyhow::Error {
    anyhow!(
        "{}",
        i18n::t_or(
            "host.error.entry_missing_required_callback_apis",
            "Entry script is missing required Callback APIs: {err}",
        )
        .replace("{err}", &format!("{label}: {err}"))
    )
}

fn to_lua_event_table(lua: &Lua, event: &InputEvent) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    match event {
        InputEvent::Action(name) => {
            table.set("type", "action")?;
            table.set("name", name.as_str())?;
        }
        InputEvent::Key(name) => {
            table.set("type", "key")?;
            table.set("name", name.as_str())?;
        }
        InputEvent::Resize { width, height } => {
            table.set("type", "resize")?;
            table.set("width", *width)?;
            table.set("height", *height)?;
        }
        InputEvent::Tick { dt_ms } => {
            table.set("type", "tick")?;
            table.set("dt_ms", *dt_ms)?;
        }
        InputEvent::Quit => {
            table.set("type", "quit")?;
        }
    }
    Ok(table)
}

fn json_to_lua_value(lua: &Lua, value: &JsonValue) -> Result<Value> {
    match value {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(value) => Ok(Value::Boolean(*value)),
        JsonValue::Number(value) => {
            if let Some(value) = value.as_i64() {
                Ok(Value::Integer(value))
            } else if let Some(value) = value.as_f64() {
                Ok(Value::Number(value))
            } else {
                Err(anyhow!("unsupported json number"))
            }
        }
        JsonValue::String(value) => Ok(Value::String(
            lua.create_string(value).map_err(anyhow_lua_error)?,
        )),
        JsonValue::Array(items) => {
            let table = lua.create_table().map_err(anyhow_lua_error)?;
            for (index, item) in items.iter().enumerate() {
                table
                    .set(index + 1, json_to_lua_value(lua, item)?)
                    .map_err(anyhow_lua_error)?;
            }
            Ok(Value::Table(table))
        }
        JsonValue::Object(object) => {
            let table = lua.create_table().map_err(anyhow_lua_error)?;
            for (key, item) in object {
                table
                    .set(key.as_str(), json_to_lua_value(lua, item)?)
                    .map_err(anyhow_lua_error)?;
            }
            Ok(Value::Table(table))
        }
    }
}

fn lua_value_to_json(value: &Value) -> Result<JsonValue> {
    match value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(value) => Ok(JsonValue::Bool(*value)),
        Value::Integer(value) => Ok(JsonValue::Number(Number::from(*value))),
        Value::Number(value) => Number::from_f64(*value)
            .map(JsonValue::Number)
            .ok_or_else(|| anyhow!("cannot convert non-finite lua number to json")),
        Value::String(value) => Ok(JsonValue::String(
            value.to_str().map_err(anyhow_lua_error)?.to_string(),
        )),
        Value::Table(table) => lua_table_to_json(table),
        _ => Err(anyhow!("unsupported lua value for json conversion")),
    }
}

fn lua_table_to_json(table: &Table) -> Result<JsonValue> {
    let mut array_entries = Vec::new();
    let mut object_entries = Map::new();
    let mut array_only = true;

    for pair in table.clone().pairs::<Value, Value>() {
        let (key, value) = pair.map_err(anyhow_lua_error)?;
        let json_value = lua_value_to_json(&value)?;
        match key {
            Value::Integer(index) if index >= 1 => {
                array_entries.push((index as usize, json_value));
            }
            Value::String(key) => {
                array_only = false;
                object_entries.insert(
                    key.to_str().map_err(anyhow_lua_error)?.to_string(),
                    json_value,
                );
            }
            Value::Nil => {}
            _ => return Err(anyhow!("unsupported lua table key for json conversion")),
        }
    }

    if array_only {
        array_entries.sort_by_key(|(index, _)| *index);
        let contiguous = array_entries
            .iter()
            .enumerate()
            .all(|(expected, (actual, _))| *actual == expected + 1);
        if contiguous {
            return Ok(JsonValue::Array(
                array_entries.into_iter().map(|(_, value)| value).collect(),
            ));
        }
    }

    for (index, value) in array_entries {
        object_entries.insert(index.to_string(), value);
    }
    Ok(JsonValue::Object(object_entries))
}
