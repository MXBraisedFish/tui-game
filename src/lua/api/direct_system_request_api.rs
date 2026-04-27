/// 系统请求 API，游戏向宿主发送命令
/// 业务逻辑：
/// 查询函数
/// 指令函数

use std::time::Duration;

use crossterm::event;
use mlua::{Lua, Table, Value, Variadic};

use crate::core::command::RuntimeCommand;
use crate::core::key::semantic_key_source;
use crate::core::runtime::LaunchMode;
use crate::core::stats;
use crate::lua::api::common;
use crate::lua::engine::RuntimeBridges;
use crate::utils::host_log;

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    {
        let bridges = bridges.clone();
        globals.set(
            "get_launch_mode",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                let mode = match bridges.launch_mode {
                    LaunchMode::New => "new",
                    LaunchMode::Continue => "continue",
                };
                lua.create_string(mode)
                    .map(Value::String)
                    .map_err(|err| {
                        let msg = err.to_string();
                        host_log::append_host_error(
                            "host.exception.get_launch_mode_failed",
                            &[("err", &msg)],
                        );
                        mlua::Error::external(
                            crate::app::i18n::t_or(
                                "host.exception.get_launch_mode_failed",
                                "Failed to get game launch mode: {err}",
                            )
                            .replace("{err}", &msg),
                        )
                    })
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "get_best_score",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                match stats::read_runtime_best_score(&bridges.game.id) {
                    Some(value) => json_to_lua(lua, &value),
                    None => Ok(Value::Nil),
                }
                .map_err(|err| {
                    let msg = err.to_string();
                    host_log::append_host_error(
                        "host.exception.get_best_score_failed",
                        &[("err", &msg)],
                    );
                    mlua::Error::external(
                        crate::app::i18n::t_or(
                            "host.exception.get_best_score_failed",
                            "Failed to get stored best score data: {err}",
                        )
                        .replace("{err}", &msg),
                    )
                })
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "request_exit",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                push_command(&bridges, RuntimeCommand::ExitGame).map_err(|err| {
                    let msg = err.to_string();
                    host_log::append_host_error(
                        "host.exception.request_exit_invalid",
                        &[("err", &msg)],
                    );
                    mlua::Error::external(
                        crate::app::i18n::t_or(
                            "host.exception.request_exit_invalid",
                            "Invalid exit request: {err}",
                        )
                        .replace("{err}", &msg),
                    )
                })
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "request_skip_event_queue",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                push_command(&bridges, RuntimeCommand::SkipEventQueue)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "request_clear_event_queue",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                push_command(&bridges, RuntimeCommand::ClearEventQueue)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "request_render",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                push_command(&bridges, RuntimeCommand::RenderNow).map_err(|err| {
                    let msg = err.to_string();
                    host_log::append_host_error(
                        "host.exception.request_render_invalid",
                        &[("err", &msg)],
                    );
                    mlua::Error::external(
                        crate::app::i18n::t_or(
                            "host.exception.request_render_invalid",
                            "Invalid render request: {err}",
                        )
                        .replace("{err}", &msg),
                    )
                })
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "request_save_best_score",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                if !bridges.game.has_best_score {
                    let msg = format!("best_none=null for {}", bridges.game.id);
                    host_log::append_host_error(
                        "host.exception.request_save_best_score_invalid",
                        &[("err", &msg)],
                    );
                    return Err(mlua::Error::external(
                        crate::app::i18n::t_or(
                            "host.exception.request_save_best_score_invalid",
                            "Invalid best score save request: {err}",
                        )
                        .replace("{err}", &msg),
                    ));
                }
                push_command(&bridges, RuntimeCommand::SaveBestScore).map_err(|err| {
                    let msg = err.to_string();
                    host_log::append_host_error(
                        "host.exception.request_save_best_score_invalid",
                        &[("err", &msg)],
                    );
                    mlua::Error::external(
                        crate::app::i18n::t_or(
                            "host.exception.request_save_best_score_invalid",
                            "Invalid best score save request: {err}",
                        )
                        .replace("{err}", &msg),
                    )
                })
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "request_save_game",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                if !bridges.game.save {
                    let msg = format!("save=false for {}", bridges.game.id);
                    host_log::append_host_error(
                        "host.exception.request_save_game_invalid",
                        &[("err", &msg)],
                    );
                    return Err(mlua::Error::external(
                        crate::app::i18n::t_or(
                            "host.exception.request_save_game_invalid",
                            "Invalid game save request: {err}",
                        )
                        .replace("{err}", &msg),
                    ));
                }
                push_command(&bridges, RuntimeCommand::SaveGame).map_err(|err| {
                    let msg = err.to_string();
                    host_log::append_host_error(
                        "host.exception.request_save_game_invalid",
                        &[("err", &msg)],
                    );
                    mlua::Error::external(
                        crate::app::i18n::t_or(
                            "host.exception.request_save_game_invalid",
                            "Invalid game save request: {err}",
                        )
                        .replace("{err}", &msg),
                    )
                })
            })?,
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
