use std::collections::BTreeMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use chrono::{Datelike, Local, NaiveDate, TimeZone, Timelike};
use mlua::{Lua, Table, Value, Variadic};

use crate::app::i18n;
use crate::lua::api::common;
use crate::lua::engine::RuntimeBridges;
use crate::utils::host_log;

const MAX_TIMERS: usize = 64;

#[derive(Default)]
pub struct TimerStore {
    next_id: u64,
    timers: BTreeMap<String, TimerEntry>,
}

#[derive(Clone)]
struct TimerEntry {
    id: String,
    note: String,
    duration_ms: u64,
    elapsed_ms: u64,
    started_at: Option<Instant>,
    status: TimerStatus,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum TimerStatus {
    Init,
    Running,
    Pause,
    Completed,
}

impl TimerStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::Running => "running",
            Self::Pause => "pause",
            Self::Completed => "completed",
        }
    }
}

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    {
        globals.set(
            "now",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                current_timestamp_millis()
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "running_time",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                i64::try_from(bridges.started_at.elapsed().as_millis())
                    .map_err(|err| running_time_failed_error(&err.to_string()))
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "timer_create",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_arg_count_range(&args, 1, 2)?;
                let delay_ms = common::expect_i64_arg(&args, 0, "delay_ms")?;
                if delay_ms <= 0 {
                    return Err(timer_duration_must_be_positive_error());
                }
                let note = common::expect_optional_string_arg(&args, 1, "note")?;
                let mut store = timer_store(&bridges)
                    .map_err(|err| create_timer_failed_error(&err.to_string()))?;
                if store.timers.len() >= MAX_TIMERS {
                    return Err(timer_limit_reached_error());
                }
                store.next_id += 1;
                let id = format!("timer_{}", store.next_id);
                store.timers.insert(
                    id.clone(),
                    TimerEntry {
                        id: id.clone(),
                        note: note.unwrap_or_default(),
                        duration_ms: delay_ms as u64,
                        elapsed_ms: 0,
                        started_at: None,
                        status: TimerStatus::Init,
                    },
                );
                lua.create_string(&id)
                    .map(Value::String)
                    .map_err(|err| create_timer_failed_error(&err.to_string()))
            })?,
        )?;
    }

    install_timer_mutator(
        lua,
        &globals,
        "timer_start",
        bridges.clone(),
        start_timer_failed_error,
        |timer| {
            normalize_timer(timer);
            if timer.status == TimerStatus::Init {
                timer.started_at = Some(Instant::now());
                timer.status = TimerStatus::Running;
            }
        },
    )?;

    install_timer_mutator(
        lua,
        &globals,
        "timer_pause",
        bridges.clone(),
        pause_timer_failed_error,
        |timer| {
            normalize_timer(timer);
            if timer.status == TimerStatus::Running {
                timer.elapsed_ms = current_elapsed(timer);
                timer.started_at = None;
                timer.status = TimerStatus::Pause;
            }
        },
    )?;

    install_timer_mutator(
        lua,
        &globals,
        "timer_resume",
        bridges.clone(),
        resume_timer_failed_error,
        |timer| {
            normalize_timer(timer);
            if timer.status == TimerStatus::Pause {
                timer.started_at = Some(Instant::now());
                timer.status = TimerStatus::Running;
            }
        },
    )?;

    install_timer_mutator(
        lua,
        &globals,
        "timer_reset",
        bridges.clone(),
        reset_timer_failed_error,
        |timer| {
            timer.elapsed_ms = 0;
            timer.started_at = None;
            timer.status = TimerStatus::Init;
        },
    )?;

    install_timer_mutator(
        lua,
        &globals,
        "timer_restart",
        bridges.clone(),
        start_timer_failed_error,
        |timer| {
            timer.elapsed_ms = 0;
            timer.started_at = Some(Instant::now());
            timer.status = TimerStatus::Running;
        },
    )?;

    {
        let bridges = bridges.clone();
        globals.set(
            "timer_kill",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let id = common::expect_string_arg(&args, 0, "id")?;
                let mut store = timer_store(&bridges)
                    .map_err(|err| kill_timer_failed_error(&err.to_string()))?;
                if store.timers.remove(&id).is_none() {
                    return Err(timer_not_found_error(&id));
                }
                Ok(())
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "set_timer_note",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 2)?;
                let id = common::expect_string_arg(&args, 0, "id")?;
                let note = common::expect_string_arg(&args, 1, "note")?;
                let mut store = timer_store(&bridges)
                    .map_err(|err| timer_info_failed_error(&id, &err.to_string()))?;
                let timer = get_timer_mut(&mut store, &id)?;
                timer.note = note;
                Ok(())
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "get_timer_list",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                let mut store = timer_store(&bridges)
                    .map_err(|err| get_timer_list_failed_error(&err.to_string()))?;
                let arr = lua.create_table()?;
                for (idx, timer) in store.timers.values_mut().enumerate() {
                    normalize_timer(timer);
                    arr.set(idx + 1, build_timer_info(lua, timer)?)?
                }
                Ok(arr)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "get_timer_info",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let id = common::expect_string_arg(&args, 0, "id")?;
                let mut store = timer_store(&bridges)
                    .map_err(|err| timer_info_failed_error(&id, &err.to_string()))?;
                let timer = get_timer_mut(&mut store, &id)?;
                normalize_timer(timer);
                build_timer_info(lua, timer)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "get_timer_status",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let id = common::expect_string_arg(&args, 0, "id")?;
                let mut store = timer_store(&bridges)
                    .map_err(|err| timer_info_failed_error(&id, &err.to_string()))?;
                let timer = get_timer_mut(&mut store, &id)?;
                normalize_timer(timer);
                lua.create_string(timer.status.as_str())
                    .map(Value::String)
                    .map_err(|err| timer_info_failed_error(&id, &err.to_string()))
            })?,
        )?;
    }

    install_timer_getter(
        lua,
        &globals,
        "get_timer_elapsed",
        bridges.clone(),
        timer_info_failed_error,
        |timer| Value::Integer(current_elapsed(timer) as i64),
    )?;

    install_timer_getter(
        lua,
        &globals,
        "get_timer_remaining",
        bridges.clone(),
        timer_info_failed_error,
        |timer| Value::Integer(current_remaining(timer) as i64),
    )?;

    install_timer_getter(
        lua,
        &globals,
        "get_timer_duration",
        bridges.clone(),
        timer_info_failed_error,
        |timer| Value::Integer(timer.duration_ms as i64),
    )?;

    install_timer_getter(
        lua,
        &globals,
        "is_timer_completed",
        bridges.clone(),
        timer_info_failed_error,
        |timer| (timer.status == TimerStatus::Completed).into_lua(),
    )?;

    {
        let bridges = bridges.clone();
        globals.set(
            "is_timer_exists",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let id = common::expect_string_arg(&args, 0, "id")?;
                let store = timer_store(&bridges)
                    .map_err(|err| timer_exists_check_failed_error(&id, &err.to_string()))?;
                Ok(store.timers.contains_key(&id))
            })?,
        )?;
    }

    globals.set(
        "get_current_year",
        lua.create_function(move |_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 0)?;
            Ok(current_local_datetime()?.year())
        })?,
    )?;
    globals.set(
        "get_current_month",
        lua.create_function(move |_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 0)?;
            Ok(current_local_datetime()?.month() as i64)
        })?,
    )?;
    globals.set(
        "get_current_day",
        lua.create_function(move |_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 0)?;
            Ok(current_local_datetime()?.day() as i64)
        })?,
    )?;
    globals.set(
        "get_current_hour",
        lua.create_function(move |_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 0)?;
            Ok(current_local_datetime()?.hour() as i64)
        })?,
    )?;
    globals.set(
        "get_current_minute",
        lua.create_function(move |_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 0)?;
            Ok(current_local_datetime()?.minute() as i64)
        })?,
    )?;
    globals.set(
        "get_current_second",
        lua.create_function(move |_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 0)?;
            Ok(current_local_datetime()?.second() as i64)
        })?,
    )?;
    globals.set(
        "timestamp_to_date",
        lua.create_function(move |_, args: Variadic<Value>| {
            common::expect_arg_count_range(&args, 1, 2)?;
            let timestamp = common::expect_i64_arg(&args, 0, "timestamp")?;
            if timestamp < 0 {
                return Err(timestamp_must_be_non_negative_error());
            }
            let format = common::expect_optional_string_arg(&args, 1, "format")?.unwrap_or_else(|| {
                "{year}-{month}-{day} {hour}:{minute}:{second}".to_string()
            });
            if !contains_any_datetime_placeholder(&format) {
                return Err(date_string_missing_required_parameters_error());
            }
            format_timestamp(timestamp, &format)
        })?,
    )?;
    globals.set(
        "date_to_timestamp",
        lua.create_function(move |_, args: Variadic<Value>| {
            common::expect_arg_count_range(&args, 0, 6)?;
            let year = common::expect_optional_i64_arg(&args, 0, "year")?.unwrap_or(2000);
            let month = common::expect_optional_i64_arg(&args, 1, "month")?.unwrap_or(1);
            let day = common::expect_optional_i64_arg(&args, 2, "day")?.unwrap_or(1);
            let hour = common::expect_optional_i64_arg(&args, 3, "hour")?.unwrap_or(0);
            let minute = common::expect_optional_i64_arg(&args, 4, "minute")?.unwrap_or(0);
            let second = common::expect_optional_i64_arg(&args, 5, "second")?.unwrap_or(0);
            build_timestamp(year, month, day, hour, minute, second)
        })?,
    )?;

    Ok(())
}

trait IntoLuaValue {
    fn into_lua(self) -> mlua::Value;
}

impl IntoLuaValue for bool {
    fn into_lua(self) -> mlua::Value {
        Value::Boolean(self)
    }
}

fn install_timer_mutator<F, E>(
    lua: &Lua,
    globals: &Table,
    name: &'static str,
    bridges: RuntimeBridges,
    error_factory: E,
    mutator: F,
) -> mlua::Result<()>
where
    F: Fn(&mut TimerEntry) + Clone + Send + 'static,
    E: Fn(&str) -> mlua::Error + Clone + Send + 'static,
{
    let apply = mutator.clone();
    let make_error = error_factory.clone();
    globals.set(
        name,
        lua.create_function(move |_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 1)?;
            let id = common::expect_string_arg(&args, 0, "id")?;
            let mut store = timer_store(&bridges).map_err(|err| make_error(&err.to_string()))?;
            let timer = get_timer_mut(&mut store, &id)?;
            apply(timer);
            Ok(())
        })?,
    )?;
    Ok(())
}

fn install_timer_getter<F, E>(
    lua: &Lua,
    globals: &Table,
    name: &'static str,
    bridges: RuntimeBridges,
    error_factory: E,
    getter: F,
) -> mlua::Result<()>
where
    F: Fn(&TimerEntry) -> Value + Clone + Send + 'static,
    E: Fn(&str, &str) -> mlua::Error + Clone + Send + 'static,
{
    let get = getter.clone();
    let make_error = error_factory.clone();
    globals.set(
        name,
        lua.create_function(move |_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 1)?;
            let id = common::expect_string_arg(&args, 0, "id")?;
            let mut store = timer_store(&bridges).map_err(|err| make_error(&id, &err.to_string()))?;
            let timer = get_timer_mut(&mut store, &id)?;
            normalize_timer(timer);
            Ok(get(timer))
        })?,
    )?;
    Ok(())
}

fn timer_store<'a>(bridges: &'a RuntimeBridges) -> mlua::Result<std::sync::MutexGuard<'a, TimerStore>> {
    bridges
        .timers
        .lock()
        .map_err(|err| mlua::Error::external(err.to_string()))
}

fn get_timer_mut<'a>(store: &'a mut TimerStore, id: &str) -> mlua::Result<&'a mut TimerEntry> {
    store
        .timers
        .get_mut(id)
        .ok_or_else(|| timer_not_found_error(id))
}

fn normalize_timer(timer: &mut TimerEntry) {
    if timer.status == TimerStatus::Running {
        let elapsed = current_elapsed(timer);
        if elapsed >= timer.duration_ms {
            timer.elapsed_ms = timer.duration_ms;
            timer.started_at = None;
            timer.status = TimerStatus::Completed;
        }
    }
}

fn current_elapsed(timer: &TimerEntry) -> u64 {
    match timer.status {
        TimerStatus::Init => 0,
        TimerStatus::Pause | TimerStatus::Completed => timer.elapsed_ms.min(timer.duration_ms),
        TimerStatus::Running => {
            let delta = timer
                .started_at
                .map(|started| started.elapsed().as_millis() as u64)
                .unwrap_or(0);
            (timer.elapsed_ms + delta).min(timer.duration_ms)
        }
    }
}

fn current_remaining(timer: &TimerEntry) -> u64 {
    timer.duration_ms.saturating_sub(current_elapsed(timer))
}

fn build_timer_info(lua: &Lua, timer: &TimerEntry) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("id", timer.id.as_str())?;
    table.set("note", timer.note.as_str())?;
    table.set("status", timer.status.as_str())?;
    table.set("elapsed", current_elapsed(timer) as i64)?;
    table.set("remaining", current_remaining(timer) as i64)?;
    table.set("duration", timer.duration_ms as i64)?;
    Ok(table)
}

fn format_timestamp(timestamp_ms: i64, format: &str) -> mlua::Result<String> {
    let Some(local_dt) = Local.timestamp_millis_opt(timestamp_ms).single() else {
        return Err(system_time_failed_error("invalid timestamp"));
    };
    Ok(format
        .replace("{year}", &format!("{:04}", local_dt.year()))
        .replace("{month}", &format!("{:02}", local_dt.month()))
        .replace("{day}", &format!("{:02}", local_dt.day()))
        .replace("{hour}", &format!("{:02}", local_dt.hour()))
        .replace("{minute}", &format!("{:02}", local_dt.minute()))
        .replace("{second}", &format!("{:02}", local_dt.second())))
}

fn build_timestamp(
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
) -> mlua::Result<Value> {
    let year = i32::try_from(year).map_err(|_| date_conversion_failed_error("year out of range"))?;
    let month =
        u32::try_from(month).map_err(|_| date_conversion_failed_error("month out of range"))?;
    let day = u32::try_from(day).map_err(|_| date_conversion_failed_error("day out of range"))?;
    let hour =
        u32::try_from(hour).map_err(|_| date_conversion_failed_error("hour out of range"))?;
    let minute = u32::try_from(minute)
        .map_err(|_| date_conversion_failed_error("minute out of range"))?;
    let second = u32::try_from(second)
        .map_err(|_| date_conversion_failed_error("second out of range"))?;

    let naive_date = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| date_conversion_failed_error("invalid date components"))?;
    let naive = naive_date
        .and_hms_opt(hour, minute, second)
        .ok_or_else(|| date_conversion_failed_error("invalid time components"))?;
    let local_dt = Local
        .from_local_datetime(&naive)
        .single()
        .ok_or_else(|| date_conversion_failed_error("ambiguous or invalid local datetime"))?;
    Ok(Value::Integer(local_dt.timestamp_millis()))
}

fn current_timestamp_millis() -> mlua::Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| system_timestamp_failed_error(&err.to_string()))?;
    i64::try_from(duration.as_millis()).map_err(|err| system_timestamp_failed_error(&err.to_string()))
}

fn current_local_datetime() -> mlua::Result<chrono::DateTime<Local>> {
    let timestamp = current_timestamp_millis().map_err(|err| system_time_failed_error(&err.to_string()))?;
    Local
        .timestamp_millis_opt(timestamp)
        .single()
        .ok_or_else(|| system_time_failed_error("invalid local time"))
}

fn contains_any_datetime_placeholder(format: &str) -> bool {
    ["{year}", "{month}", "{day}", "{hour}", "{minute}", "{second}"]
        .iter()
        .any(|placeholder| format.contains(placeholder))
}

fn running_time_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.running_time_failed", &[("err", err)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.running_time_failed",
            "Failed to get current game runtime: {err}",
        )
        .replace("{err}", err),
    )
}

fn timer_duration_must_be_positive_error() -> mlua::Error {
    host_log::append_host_error("host.exception.timer_duration_must_be_positive", &[]);
    mlua::Error::external(i18n::t_or(
        "host.exception.timer_duration_must_be_positive",
        "Timer duration must be a positive integer",
    ))
}

fn timer_not_found_error(id: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.timer_not_found", &[("id", id)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.timer_not_found",
            "Timer with specified ID does not exist: {id}",
        )
        .replace("{id}", id),
    )
}

fn create_timer_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.create_timer_failed", &[("err", err)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.create_timer_failed",
            "Failed to create timer: {err}",
        )
        .replace("{err}", err),
    )
}

fn timer_limit_reached_error() -> mlua::Error {
    host_log::append_host_error("host.exception.timer_limit_reached", &[]);
    mlua::Error::external(i18n::t_or(
        "host.exception.timer_limit_reached",
        "Timer limit of 64 has been reached",
    ))
}

fn start_timer_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.start_timer_failed", &[("err", err)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.start_timer_failed",
            "Failed to start timer with specified ID: {err}",
        )
        .replace("{err}", err),
    )
}

fn pause_timer_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.pause_timer_failed", &[("err", err)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.pause_timer_failed",
            "Failed to pause timer with specified ID: {err}",
        )
        .replace("{err}", err),
    )
}

fn resume_timer_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.resume_timer_failed", &[("err", err)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.resume_timer_failed",
            "Failed to resume timer with specified ID: {err}",
        )
        .replace("{err}", err),
    )
}

fn reset_timer_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.reset_timer_failed", &[("err", err)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.reset_timer_failed",
            "Failed to reset timer with specified ID: {err}",
        )
        .replace("{err}", err),
    )
}

fn kill_timer_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.kill_timer_failed", &[("err", err)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.kill_timer_failed",
            "Failed to kill timer with specified ID: {err}",
        )
        .replace("{err}", err),
    )
}

fn get_timer_list_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.get_timer_list_failed", &[("err", err)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.get_timer_list_failed",
            "Failed to get current timer list: {err}",
        )
        .replace("{err}", err),
    )
}

fn timer_info_failed_error(id: &str, _err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.timer_info_failed", &[("id", id)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.timer_info_failed",
            "Failed to get info for timer with specified ID: {id}",
        )
        .replace("{id}", id),
    )
}

fn timer_exists_check_failed_error(id: &str, _err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.timer_exists_check_failed", &[("id", id)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.timer_exists_check_failed",
            "Failed to check existence of timer with specified ID: {id}",
        )
        .replace("{id}", id),
    )
}

fn system_timestamp_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.system_timestamp_failed", &[("err", err)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.system_timestamp_failed",
            "Failed to get system timestamp: {err}",
        )
        .replace("{err}", err),
    )
}

fn system_time_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.system_time_failed", &[("err", err)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.system_time_failed",
            "Failed to get system time: {err}",
        )
        .replace("{err}", err),
    )
}

fn timestamp_must_be_non_negative_error() -> mlua::Error {
    host_log::append_host_error("host.exception.timestamp_must_be_non_negative", &[]);
    mlua::Error::external(i18n::t_or(
        "host.exception.timestamp_must_be_non_negative",
        "Timestamp must be a non-negative integer",
    ))
}

fn date_string_missing_required_parameters_error() -> mlua::Error {
    host_log::append_host_error("host.exception.date_string_missing_required_parameters", &[]);
    mlua::Error::external(i18n::t_or(
        "host.exception.date_string_missing_required_parameters",
        "Date string missing required parameters",
    ))
}

fn date_conversion_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.date_conversion_failed", &[("err", err)]);
    mlua::Error::external(i18n::t_or(
        "host.exception.date_conversion_failed",
        "Date conversion failed: {err}",
    )
    .replace("{err}", err))
}
