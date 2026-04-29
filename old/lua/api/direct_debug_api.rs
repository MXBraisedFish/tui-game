// 调试 API，提供日志输出和游戏信息查询功能。仅在游戏的 debug_enabled 为真时有效，否则调用无效果。包括 debug_log, debug_warn, debug_error, debug_print, clear_debug_log, get_game_uid, get_game_info, get_key 等函数

use std::fs; // 写日志文件

use mlua::{Lua, Table, Value, Variadic}; // Lua 类型

use crate::app::i18n; // 国际化
use crate::game::action::ActionBinding; // 动作绑定
use crate::lua::api::common; // 参数校验
use crate::lua::engine::RuntimeBridges; // 运行时桥接
use crate::utils::path_utils; // 日志路径

// 向全局表注入所有调试函数：debug_log, debug_warn, debug_error, debug_print, clear_debug_log, get_game_uid, get_game_info, get_key
pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    {
        let bridges = bridges.clone();
        globals.set(
            "debug_log",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                if !is_debug_enabled(&bridges) {
                    return Ok(());
                }
                write_log_line(
                    &bridges,
                    &i18n::t_or("debug.title.log", "日志"),
                    &stringify_value(&args[0]),
                )
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "debug_warn",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                if !is_debug_enabled(&bridges) {
                    return Ok(());
                }
                write_log_line(
                    &bridges,
                    &i18n::t_or("debug.title.warning", "警告"),
                    &stringify_value(&args[0]),
                )
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "debug_error",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                if !is_debug_enabled(&bridges) {
                    return Ok(());
                }
                write_log_line(
                    &bridges,
                    &i18n::t_or("debug.title.error", "异常"),
                    &stringify_value(&args[0]),
                )
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "debug_print",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 2)?;
                let title = common::expect_string_arg(&args, 0, "title")?;
                let message = args.get(1).cloned().unwrap_or(Value::Nil);
                if !is_debug_enabled(&bridges) {
                    return Ok(());
                }
                write_log_line(&bridges, &title, &stringify_value(&message))
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "clear_debug_log",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                if !is_debug_enabled(&bridges) {
                    return Ok(());
                }
                let path = debug_log_path(&bridges)?;
                path_utils::ensure_parent_dir(&path).map_err(mlua::Error::external)?;
                fs::write(&path, "")
                    .map_err(|_| {
                        mlua::Error::external(crate::app::i18n::t_or(
                            "host.error.log_write_failed",
                            "Failed to write log.",
                        ))
                    })?;
                Ok(())
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "get_game_uid",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                if !is_debug_enabled(&bridges) {
                    return Ok(Value::Nil);
                }
                Ok(Value::String(lua.create_string(bridges.game.id.as_str())?))
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "get_game_info",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                if !is_debug_enabled(&bridges) {
                    return Ok(Value::Nil);
                }
                Ok(Value::Table(build_game_info(lua, &bridges)?))
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "get_key",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_arg_count_range(&args, 0, 1)?;
                let requested_key = common::expect_optional_string_arg(&args, 0, "key")?;
                Ok(Value::Table(build_key_info_table(
                    lua,
                    &bridges,
                    requested_key.as_deref(),
                )?))
            })?,
        )?;
    }

    Ok(())
}

// 检查 package.debug_enabled 标志
pub(crate) fn is_debug_enabled(bridges: &RuntimeBridges) -> bool {
    bridges
        .game
        .package
        .as_ref()
        .map(|package| package.debug_enabled)
        .unwrap_or(false)
}

// 返回 log_dir/{game.id}.txt
pub(crate) fn debug_log_path(bridges: &RuntimeBridges) -> mlua::Result<std::path::PathBuf> {
    Ok(path_utils::log_dir()
        .map_err(mlua::Error::external)?
        .join(format!("{}.txt", bridges.game.id)))
}

// 追加一行日志到调试日志文件
pub(crate) fn write_log_line(
    bridges: &RuntimeBridges,
    title: &str,
    message: &str,
) -> mlua::Result<()> {
    let path = debug_log_path(bridges)?;
    path_utils::ensure_parent_dir(&path).map_err(mlua::Error::external)?;
    let line = format!("[{}] {}\n", title, message);
    let mut existing = fs::read_to_string(&path).unwrap_or_default();
    existing.push_str(&line);
    fs::write(&path, existing).map_err(|_| {
        mlua::Error::external(crate::app::i18n::t_or(
            "host.error.log_write_failed",
            "Failed to write log.",
        ))
    })?;
    Ok(())
}

// 将 GameDescriptor 转换为 Lua 表（包含包名、作者、动作、图标等）
fn build_game_info(lua: &Lua, bridges: &RuntimeBridges) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let package = bridges.game.package.as_ref();

    table.set("uid", bridges.game.id.as_str())?;
    table.set(
        "package",
        package
            .map(|value| value.package_name.as_str())
            .unwrap_or_default(),
    )?;
    set_optional_string(
        &table,
        "mod_name",
        package.and_then(|value| value.mod_name.as_deref()),
    )?;
    set_optional_string(&table, "introduction", bridges.game.introduction.as_deref())?;
    table.set("author", bridges.game.author.as_str())?;
    table.set("game_name", bridges.game.name.as_str())?;
    table.set("description", bridges.game.description.as_str())?;
    table.set("detail", bridges.game.detail.as_str())?;
    set_json_value(lua, &table, "icon", bridges.game.icon.as_ref())?;
    set_json_value(lua, &table, "banner", bridges.game.banner.as_ref())?;
    set_json_value(lua, &table, "api", bridges.game.api.as_ref())?;
    table.set("entry", bridges.game.entry.as_str())?;
    table.set("save", bridges.game.save)?;
    set_optional_string(&table, "best_none", bridges.game.best_none.as_deref())?;
    set_optional_u16(&table, "min_width", bridges.game.min_width)?;
    set_optional_u16(&table, "min_height", bridges.game.min_height)?;
    table.set("write", bridges.game.write)?;
    table.set("case_sensitive", bridges.game.case_sensitive)?;

    let actions = lua.create_table()?;
    for (name, binding) in &bridges.game.actions {
        actions.set(
            name.as_str(),
            build_action_binding_table(lua, binding, None)?,
        )?;
    }
    table.set("actions", actions)?;

    let runtime = lua.create_table()?;
    runtime.set("target_fps", bridges.game.target_fps)?;
    table.set("runtime", runtime)?;

    Ok(table)
}

// 构建按键信息表：若指定 requested_key 则仅返回该动作的绑定；否则返回所有动作的绑定信息
fn build_key_info_table(
    lua: &Lua,
    bridges: &RuntimeBridges,
    requested_key: Option<&str>,
) -> mlua::Result<Table> {
    let table = lua.create_table()?;

    for (semantic_key, binding) in &bridges.game.actions {
        if requested_key.is_some_and(|key| key != semantic_key) {
            continue;
        }
        let default_binding = bridges.game.default_actions.get(semantic_key);
        table.set(
            semantic_key.as_str(),
            build_action_binding_table(lua, default_binding.unwrap_or(binding), Some(binding))?,
        )?;
    }

    Ok(table)
}

// 构建单个动作绑定的 Lua 表，包含 key（字符串或数组）、key_name、可选的 key_user（用户覆盖的键）
fn build_action_binding_table(
    lua: &Lua,
    binding: &ActionBinding,
    user_binding: Option<&ActionBinding>,
) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    set_key_value(lua, &table, "key", &binding.keys())?;
    table.set("key_name", binding.key_name())?;
    if let Some(user_binding) = user_binding {
        set_key_value(lua, &table, "key_user", &user_binding.keys())?;
    }
    Ok(table)
}

// 将 Vec<String> 设置为字段：若长度为1则设为字符串，否则设为数组
fn set_key_value(lua: &Lua, table: &Table, field: &str, keys: &[String]) -> mlua::Result<()> {
    if keys.len() == 1 {
        table.set(field, keys[0].clone())
    } else {
        let arr = lua.create_table()?;
        for (idx, key) in keys.iter().enumerate() {
            arr.set(idx + 1, key.as_str())?;
        }
        table.set(field, arr)
    }
}

// 辅助设置可选字段，为空时设为 nil
fn set_optional_string(table: &Table, key: &str, value: Option<&str>) -> mlua::Result<()> {
    match value {
        Some(value) => table.set(key, value),
        None => table.set(key, Value::Nil),
    }
}

// 辅助设置可选字段，为空时设为 nil
fn set_optional_u16(table: &Table, key: &str, value: Option<u16>) -> mlua::Result<()> {
    match value {
        Some(value) => table.set(key, value),
        None => table.set(key, Value::Nil),
    }
}

// 辅助设置可选字段，为空时设为 nil
fn set_json_value(
    lua: &Lua,
    table: &Table,
    key: &str,
    value: Option<&serde_json::Value>,
) -> mlua::Result<()> {
    match value {
        Some(value) => table.set(key, json_to_lua(lua, value)?),
        None => table.set(key, Value::Nil),
    }
}

// 将 serde_json 值递归转换为 Lua 值
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
            let obj = lua.create_table()?;
            for (key, item) in map {
                obj.set(key.as_str(), json_to_lua(lua, item)?)?;
            }
            Ok(Value::Table(obj))
        }
    }
}

// 将任意 Lua 值转为字符串（用于日志消息）
fn stringify_value(value: &Value) -> String {
    match value {
        Value::Nil => "nil".to_string(),
        Value::Boolean(value) => value.to_string(),
        Value::LightUserData(_) => "<lightuserdata>".to_string(),
        Value::Integer(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value
            .to_str()
            .map(|value| value.to_string())
            .unwrap_or_else(|_| "<string>".to_string()),
        Value::Table(_) => "<table>".to_string(),
        Value::Function(_) => "<function>".to_string(),
        Value::Thread(_) => "<thread>".to_string(),
        Value::UserData(_) => "<userdata>".to_string(),
        Value::Error(err) => err.to_string(),
        Value::Other(_) => "<other>".to_string(),
    }
}
