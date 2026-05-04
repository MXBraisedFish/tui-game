//! 直用式时间处理 API 公开

use chrono::{Datelike, Timelike};
use mlua::{Lua, Value, Variadic};

use super::scope::ApiScope;
use super::timer_support::date_time;
use super::timer_support::timer_table;
use super::validation::argument;
use crate::host_engine::boot::preload::lua_runtime::HostLuaBridge;

const DEFAULT_DATE_FORMAT: &str = "{year}-{month}-{day} {hour}:{minute}:{second}";

/// 安装时间处理 API。
pub fn install(lua: &Lua, api_scope: ApiScope, host_bridge: HostLuaBridge) -> mlua::Result<()> {
    if !api_scope.allows_timer() {
        return Ok(());
    }

    let globals = lua.globals();
    install_running_time(lua, &globals, host_bridge.clone())?;
    install_timer_create(lua, &globals, host_bridge.clone())?;
    install_timer_start(lua, &globals, host_bridge.clone())?;
    install_timer_pause(lua, &globals, host_bridge.clone())?;
    install_timer_resume(lua, &globals, host_bridge.clone())?;
    install_timer_reset(lua, &globals, host_bridge.clone())?;
    install_timer_restart(lua, &globals, host_bridge.clone())?;
    install_timer_kill(lua, &globals, host_bridge.clone())?;
    install_set_timer_note(lua, &globals, host_bridge.clone())?;
    install_get_timer_list(lua, &globals, host_bridge.clone())?;
    install_get_timer_info(lua, &globals, host_bridge.clone())?;
    install_get_timer_status(lua, &globals, host_bridge.clone())?;
    install_get_timer_elapsed(lua, &globals, host_bridge.clone())?;
    install_get_timer_remaining(lua, &globals, host_bridge.clone())?;
    install_get_timer_duration(lua, &globals, host_bridge.clone())?;
    install_is_timer_completed(lua, &globals, host_bridge.clone())?;
    install_is_timer_exists(lua, &globals, host_bridge.clone())?;
    install_now(lua, &globals)?;
    install_current_date_parts(lua, &globals)?;
    install_timestamp_to_date(lua, &globals)?;
    install_date_to_timestamp(lua, &globals)?;

    Ok(())
}

fn install_running_time(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "running_time",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            Ok(host_bridge.running_time_ms())
        })?,
    )
}

fn install_timer_create(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "timer_create",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 1, 2)?;
            let delay_ms = argument::expect_i64_arg(&args, 0)?;
            if delay_ms <= 0 {
                return Err(mlua::Error::external("timer duration must be positive"));
            }
            let note = argument::expect_optional_string_arg(&args, 1)?.unwrap_or_default();
            let id = host_bridge
                .with_timer_store(|timer_store| timer_store.create_timer(delay_ms as u64, note))?;
            Ok(Value::String(lua.create_string(id.as_str())?))
        })?,
    )
}

fn install_timer_start(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_timer_mutator(lua, globals, host_bridge, "timer_start", |timer| {
        timer.start()
    })
}

fn install_timer_pause(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_timer_mutator(lua, globals, host_bridge, "timer_pause", |timer| {
        timer.pause()
    })
}

fn install_timer_resume(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_timer_mutator(lua, globals, host_bridge, "timer_resume", |timer| {
        timer.resume()
    })
}

fn install_timer_reset(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_timer_mutator(lua, globals, host_bridge, "timer_reset", |timer| {
        timer.reset()
    })
}

fn install_timer_restart(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_timer_mutator(lua, globals, host_bridge, "timer_restart", |timer| {
        timer.restart()
    })
}

fn install_timer_mutator(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
    function_name: &'static str,
    mutator: fn(&mut super::timer_support::timer_store::TimerEntry),
) -> mlua::Result<()> {
    globals.set(
        function_name,
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let id = argument::expect_string_arg(&args, 0)?;
            host_bridge.with_timer_store(|timer_store| {
                let timer = timer_store.timer_mut(id.as_str())?;
                mutator(timer);
                Ok(())
            })
        })?,
    )
}

fn install_timer_kill(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "timer_kill",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let id = argument::expect_string_arg(&args, 0)?;
            host_bridge.with_timer_store(|timer_store| timer_store.kill_timer(id.as_str()))
        })?,
    )
}

fn install_set_timer_note(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "set_timer_note",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 2)?;
            let id = argument::expect_string_arg(&args, 0)?;
            let note = argument::expect_string_arg(&args, 1)?;
            host_bridge.with_timer_store(|timer_store| {
                timer_store.timer_mut(id.as_str())?.set_note(note);
                Ok(())
            })
        })?,
    )
}

fn install_get_timer_list(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_timer_list",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            host_bridge.with_timer_store(|timer_store| {
                let table = lua.create_table()?;
                for (index, timer) in timer_store.timers_mut().enumerate() {
                    table.set(index + 1, timer_table::build_timer_info_table(lua, timer)?)?;
                }
                Ok(table)
            })
        })?,
    )
}

fn install_get_timer_info(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_timer_info",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let id = argument::expect_string_arg(&args, 0)?;
            host_bridge.with_timer_store(|timer_store| {
                timer_table::build_timer_info_table(lua, timer_store.timer_mut(id.as_str())?)
            })
        })?,
    )
}

fn install_get_timer_status(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_timer_status",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let id = argument::expect_string_arg(&args, 0)?;
            host_bridge.with_timer_store(|timer_store| {
                let status = timer_store.timer_mut(id.as_str())?.status().as_str();
                Ok(Value::String(lua.create_string(status)?))
            })
        })?,
    )
}

fn install_get_timer_elapsed(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_timer_value_getter(lua, globals, host_bridge, "get_timer_elapsed", |timer| {
        Value::Integer(timer.elapsed_ms() as i64)
    })
}

fn install_get_timer_remaining(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_timer_value_getter(lua, globals, host_bridge, "get_timer_remaining", |timer| {
        Value::Integer(timer.remaining_ms() as i64)
    })
}

fn install_get_timer_duration(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_timer_value_getter(lua, globals, host_bridge, "get_timer_duration", |timer| {
        Value::Integer(timer.duration_ms as i64)
    })
}

fn install_is_timer_completed(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_timer_value_getter(lua, globals, host_bridge, "is_timer_completed", |timer| {
        Value::Boolean(timer.is_completed())
    })
}

fn install_timer_value_getter(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
    function_name: &'static str,
    getter: fn(&mut super::timer_support::timer_store::TimerEntry) -> Value,
) -> mlua::Result<()> {
    globals.set(
        function_name,
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let id = argument::expect_string_arg(&args, 0)?;
            host_bridge.with_timer_store(|timer_store| {
                let timer = timer_store.timer_mut(id.as_str())?;
                timer.normalize();
                Ok(getter(timer))
            })
        })?,
    )
}

fn install_is_timer_exists(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "is_timer_exists",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let id = argument::expect_string_arg(&args, 0)?;
            host_bridge.with_timer_store(|timer_store| Ok(timer_store.contains_timer(id.as_str())))
        })?,
    )
}

fn install_now(lua: &Lua, globals: &mlua::Table) -> mlua::Result<()> {
    globals.set(
        "now",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            date_time::current_timestamp_ms()
        })?,
    )
}

fn install_current_date_parts(lua: &Lua, globals: &mlua::Table) -> mlua::Result<()> {
    globals.set(
        "get_current_year",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            Ok(date_time::current_local_time()?.year() as i64)
        })?,
    )?;
    globals.set(
        "get_current_month",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            Ok(date_time::current_local_time()?.month() as i64)
        })?,
    )?;
    globals.set(
        "get_current_day",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            Ok(date_time::current_local_time()?.day() as i64)
        })?,
    )?;
    globals.set(
        "get_current_hour",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            Ok(date_time::current_local_time()?.hour() as i64)
        })?,
    )?;
    globals.set(
        "get_current_minute",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            Ok(date_time::current_local_time()?.minute() as i64)
        })?,
    )?;
    globals.set(
        "get_current_second",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            Ok(date_time::current_local_time()?.second() as i64)
        })?,
    )
}

fn install_timestamp_to_date(lua: &Lua, globals: &mlua::Table) -> mlua::Result<()> {
    globals.set(
        "timestamp_to_date",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 1, 2)?;
            let timestamp = argument::expect_i64_arg(&args, 0)?;
            let format_text = argument::expect_optional_string_arg(&args, 1)?
                .unwrap_or_else(|| DEFAULT_DATE_FORMAT.to_string());
            date_time::timestamp_to_date(timestamp, format_text.as_str())
        })?,
    )
}

fn install_date_to_timestamp(lua: &Lua, globals: &mlua::Table) -> mlua::Result<()> {
    globals.set(
        "date_to_timestamp",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 0, 6)?;
            let year = argument::expect_optional_i64_arg(&args, 0)?.unwrap_or(2000);
            let month = argument::expect_optional_i64_arg(&args, 1)?.unwrap_or(1);
            let day = argument::expect_optional_i64_arg(&args, 2)?.unwrap_or(1);
            let hour = argument::expect_optional_i64_arg(&args, 3)?.unwrap_or(0);
            let minute = argument::expect_optional_i64_arg(&args, 4)?.unwrap_or(0);
            let second = argument::expect_optional_i64_arg(&args, 5)?.unwrap_or(0);
            date_time::date_to_timestamp(year, month, day, hour, minute, second)
        })?,
    )
}
