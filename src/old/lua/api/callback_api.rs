// 游戏生命周期回调 API，管理 Lua 与 Rust 之间的状态传递。负责验证必需回调函数的存在、初始化游戏状态（根据启动模式加载存档）、分发事件、调用渲染和退出回调，以及持久化最佳成绩和游戏进度。属于 Lua API 的核心部分，供 engine.rs 调用

use anyhow::{Result, anyhow}; // 错误处理
use mlua::{Function, Lua, RegistryKey, Table, Value}; // Lua 虚拟机核心类型
use serde_json::{Map, Number, Value as JsonValue}; // 	JSON 与 Lua 值互转

use crate::app::i18n; // 国际化错误消息
use crate::core::event::InputEvent; // 输入事件类型
use crate::core::runtime::LaunchMode; // 启动模式
use crate::core::save as runtime_save; // 存档读写
use crate::core::stats as runtime_stats; // 最佳成绩读写
use crate::game::registry::GameDescriptor; // 游戏描述符
use crate::lua::engine::anyhow_lua_error; // Lua 错误转 anyhow
use crate::utils::host_log; // 	日志记录

// 检查 init_game, handle_event, render, exit_game 是否存在。若 has_best_score 为真则需 save_best_score；若 save 为真则需 save_game
pub(crate) fn validate_required_callbacks(lua: &Lua, game: &GameDescriptor) -> Result<()> {
    require_callback(lua, "init_game", "init_game(state)")?;
    require_callback(lua, "handle_event", "handle_event(state, event)")?;
    require_callback(lua, "render", "render(state)")?;
    require_callback(lua, "exit_game", "exit_game(state)")?;
    if game.has_best_score {
        require_save_best_score_callback(lua)?;
    }
    if game.save {
        require_save_game_callback(lua)?;
    }
    Ok(())
}

// 调用 init_game，根据模式传入继续存档或 nil，将返回的状态表存入注册表，返回键
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
    ensure_required_callback_value(&state)?;
    lua.create_registry_value(state).map_err(anyhow_lua_error)
}

// 调用 handle_event(state, event_table)，返回新的状态表（替换旧状态）
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
    ensure_required_callback_value(&new_state)?;
    lua.create_registry_value(new_state).map_err(anyhow_lua_error)
}

// 调用 render(state)
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

// 调用 exit_game(state)，返回更新后的状态键
pub(crate) fn call_exit_game(lua: &Lua, state_key: &RegistryKey) -> Result<RegistryKey> {
    let exit_game: Function = lua
        .globals()
        .get("exit_game")
        .map_err(|err| missing_entry_callback_error("exit_game(state)", &err.to_string()))?;
    let state = lua
        .registry_value::<Value>(state_key)
        .map_err(anyhow_lua_error)?;
    let new_state = exit_game.call::<Value>(state).map_err(anyhow_lua_error)?;
    ensure_required_callback_value(&new_state)?;
    lua.create_registry_value(new_state).map_err(anyhow_lua_error)
}

// 调用 save_best_score(state)，将返回的 JSON 值写入 best_scores.json（仅在 has_best_score 时）
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
        .map_err(|err| missing_save_best_score_error(&err.to_string()))?;
    let state = lua
        .registry_value::<Value>(state_key)
        .map_err(anyhow_lua_error)?;
    let value = save_best_score
        .call::<Value>(state)
        .map_err(anyhow_lua_error)?;
    ensure_required_callback_value(&value)?;
    runtime_stats::write_runtime_best_score(game.id.as_str(), &lua_value_to_json(&value)?)?;
    Ok(())
}

// 调用 save_game(state)，将返回的 JSON 值写入 saves.json 的 continue 条目（仅在 save 时）
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
        .map_err(|err| missing_save_game_error(&err.to_string()))?;
    let state = lua
        .registry_value::<Value>(state_key)
        .map_err(anyhow_lua_error)?;
    let value = save_game.call::<Value>(state).map_err(anyhow_lua_error)?;
    ensure_required_callback_value(&value)?;
    runtime_save::save_continue(game.id.as_str(), &lua_value_to_json(&value)?)?;
    Ok(())
}

// 验证全局函数存在，否则抛出错误
fn require_callback(lua: &Lua, global_name: &str, label: &str) -> Result<()> {
    let _: Function = lua
        .globals()
        .get(global_name)
        .map_err(|err| {
            anyhow!(
                "{}",
                i18n::t_or(
                    "host.error.entry_missing_required_callback_apis",
                    "Entry script is missing required Callback APIs: {err}",
                )
                .replace("{err}", &format!("{label}: {err}"))
            )
        })?;
    Ok(())
}

// 验证全局函数存在，否则抛出错误
fn require_save_best_score_callback(lua: &Lua) -> Result<()> {
    let _: Function = lua.globals().get("save_best_score").map_err(|err| {
        missing_save_best_score_error(&err.to_string())
    })?;
    Ok(())
}

// 验证全局函数存在，否则抛出错误
fn require_save_game_callback(lua: &Lua) -> Result<()> {
    let _: Function = lua.globals().get("save_game").map_err(|err| {
        missing_save_game_error(&err.to_string())
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

fn missing_save_best_score_error(_err: &str) -> anyhow::Error {
    anyhow!(
        "{}",
        i18n::t_or(
            "host.error.save_best_score_not_implemented",
            "best_none is not null, but save_best_score is not implemented.",
        )
    )
}

fn missing_save_game_error(_err: &str) -> anyhow::Error {
    anyhow!(
        "{}",
        i18n::t_or(
            "host.error.save_game_not_implemented",
            "save is true, but save_game is not implemented.",
        )
    )
}

fn ensure_required_callback_value(value: &Value) -> Result<()> {
    if matches!(value, Value::Nil) {
        host_log::append_host_error("host.error.missing_required_callback_api", &[]);
        return Err(anyhow!(
            "{}",
            i18n::t_or(
                "host.error.missing_required_callback_api",
                "Callback API did not return the required value.",
            )
        ));
    }
    Ok(())
}

// 将 Rust InputEvent 转换为 Lua 表，包含 type 字段和具体数据
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

// 递归将 serde_json 值转换为 Lua 值（null→nil，bool→bool，number→number/整数，string→string，array/object→table）
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

// 将 Lua 值转换为 serde_json 值（支持 nil, bool, number, string, table）
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

// 将 Lua 表转换为 JSON：若表键为连续整数则转为数组，否则转为对象
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
