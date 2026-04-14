use std::time::Duration;

use crossterm::event;
use mlua::{Lua, Table, Value};

use crate::core::command::RuntimeCommand;
use crate::core::key::semantic_key_source;
use crate::core::runtime::LaunchMode;
use crate::core::stats;
use crate::lua::engine::RuntimeBridges;

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    {
        let bridges = bridges.clone();
        globals.set(
            "get_launch_mode",
            lua.create_function(move |lua, ()| {
                let mode = match bridges.launch_mode {
                    LaunchMode::New => "new",
                    LaunchMode::Continue => "continue",
                };
                Ok(Value::String(lua.create_string(mode)?))
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "get_best_score",
            lua.create_function(move |lua, ()| match stats::read_runtime_best_score(&bridges.game.id)
            {
                Some(value) => json_to_lua(lua, &value),
                None => Ok(Value::Nil),
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "request_exit",
            lua.create_function(move |_, ()| push_command(&bridges, RuntimeCommand::ExitGame))?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "request_skip_event_queue",
            lua.create_function(move |_, ()| {
                push_command(&bridges, RuntimeCommand::SkipEventQueue)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "request_clear_event_queue",
            lua.create_function(move |_, ()| {
                push_command(&bridges, RuntimeCommand::ClearEventQueue)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "request_render",
            lua.create_function(move |_, ()| push_command(&bridges, RuntimeCommand::RenderNow))?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "request_save_best_score",
            lua.create_function(move |_, ()| {
                push_command(&bridges, RuntimeCommand::SaveBestScore)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "request_save_game",
            lua.create_function(move |_, ()| push_command(&bridges, RuntimeCommand::SaveGame))?,
        )?;
    }

    Ok(())
}

pub(crate) fn clear_pending_input_queue() {
    while event::poll(Duration::from_millis(0)).unwrap_or(false) {
        let _ = event::read();
    }
    semantic_key_source().clear_pending_keys();
}

fn push_command(bridges: &RuntimeBridges, command: RuntimeCommand) -> mlua::Result<()> {
    let mut commands = bridges
        .commands
        .lock()
        .map_err(|_| mlua::Error::external("command queue poisoned"))?;
    commands.push(command);
    Ok(())
}

fn json_to_lua(lua: &Lua, value: &serde_json::Value) -> mlua::Result<Value> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(value) => Ok(Value::Boolean(*value)),
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                Ok(Value::Integer(value))
            } else if let Some(value) = value.as_f64() {
                Ok(Value::Number(value))
            } else {
                Ok(Value::Nil)
            }
        }
        serde_json::Value::String(value) => Ok(Value::String(lua.create_string(value)?)),
        serde_json::Value::Array(items) => {
            let arr = lua.create_table()?;
            for (idx, item) in items.iter().enumerate() {
                arr.set(idx + 1, json_to_lua(lua, item)?)?;
            }
            Ok(Value::Table(arr))
        }
        serde_json::Value::Object(map) => {
            let obj: Table = lua.create_table()?;
            for (key, item) in map {
                obj.set(key.as_str(), json_to_lua(lua, item)?)?;
            }
            Ok(Value::Table(obj))
        }
    }
}
