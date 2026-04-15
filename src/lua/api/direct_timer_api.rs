use std::collections::BTreeMap;
use std::time::Instant;

use chrono::{Datelike, Local, NaiveDateTime, TimeZone, Timelike};
use mlua::{Lua, Table, Value};

use crate::lua::engine::RuntimeBridges;

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
            lua.create_function(move |_, ()| Ok(Local::now().timestamp_millis()))?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "running_time",
            lua.create_function(move |_, ()| Ok(bridges.started_at.elapsed().as_millis() as i64))?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "timer_create",
            lua.create_function(move |lua, (delay_ms, note): (i64, Option<String>)| {
                let mut store = timer_store(&bridges)?;
                if store.timers.len() >= MAX_TIMERS {
                    return Ok(Value::Nil);
                }
                store.next_id += 1;
                let id = format!("timer_{}", store.next_id);
                store.timers.insert(
                    id.clone(),
                    TimerEntry {
                        id: id.clone(),
                        note: note.unwrap_or_default(),
                        duration_ms: delay_ms.max(0) as u64,
                        elapsed_ms: 0,
                        started_at: None,
                        status: TimerStatus::Init,
                    },
                );
                Ok(Value::String(lua.create_string(&id)?))
            })?,
        )?;
    }

    install_timer_mutator(lua, &globals, "timer_start", bridges.clone(), |timer| {
        normalize_timer(timer);
        if timer.status == TimerStatus::Init {
            timer.started_at = Some(Instant::now());
            timer.status = TimerStatus::Running;
        }
    })?;

    install_timer_mutator(lua, &globals, "timer_pause", bridges.clone(), |timer| {
        normalize_timer(timer);
        if timer.status == TimerStatus::Running {
            timer.elapsed_ms = current_elapsed(timer);
            timer.started_at = None;
            timer.status = TimerStatus::Pause;
        }
    })?;

    install_timer_mutator(lua, &globals, "timer_resume", bridges.clone(), |timer| {
        normalize_timer(timer);
        if timer.status == TimerStatus::Pause {
            timer.started_at = Some(Instant::now());
            timer.status = TimerStatus::Running;
        }
    })?;

    install_timer_mutator(lua, &globals, "timer_reset", bridges.clone(), |timer| {
        timer.elapsed_ms = 0;
        timer.started_at = None;
        timer.status = TimerStatus::Init;
    })?;

    install_timer_mutator(lua, &globals, "timer_restart", bridges.clone(), |timer| {
        timer.elapsed_ms = 0;
        timer.started_at = Some(Instant::now());
        timer.status = TimerStatus::Running;
    })?;

    {
        let bridges = bridges.clone();
        globals.set(
            "timer_kill",
            lua.create_function(move |_, id: String| {
                let mut store = timer_store(&bridges)?;
                store.timers.remove(&id);
                Ok(())
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "set_timer_note",
            lua.create_function(move |_, (id, note): (String, String)| {
                let mut store = timer_store(&bridges)?;
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
            lua.create_function(move |lua, ()| {
                let mut store = timer_store(&bridges)?;
                let arr = lua.create_table()?;
                for (idx, timer) in store.timers.values_mut().enumerate() {
                    normalize_timer(timer);
                    arr.set(idx + 1, build_timer_info(lua, timer)?)?;
                }
                Ok(arr)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "get_timer_info",
            lua.create_function(move |lua, id: String| {
                let mut store = timer_store(&bridges)?;
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
            lua.create_function(move |lua, id: String| {
                let mut store = timer_store(&bridges)?;
                let timer = get_timer_mut(&mut store, &id)?;
                normalize_timer(timer);
                Ok(Value::String(lua.create_string(timer.status.as_str())?))
            })?,
        )?;
    }

    install_timer_getter(lua, &globals, "get_timer_elapsed", bridges.clone(), |timer| {
        Value::Integer(current_elapsed(timer) as i64)
    })?;

    install_timer_getter(lua, &globals, "get_timer_remaining", bridges.clone(), |timer| {
        Value::Integer(current_remaining(timer) as i64)
    })?;

    install_timer_getter(lua, &globals, "get_timer_duration", bridges.clone(), |timer| {
        Value::Integer(timer.duration_ms as i64)
    })?;

    install_timer_getter(lua, &globals, "get_timer_completed", bridges.clone(), |timer| {
        (timer.status == TimerStatus::Completed).into_lua()
    })?;

    {
        let bridges = bridges.clone();
        globals.set(
            "is_timer_exists",
            lua.create_function(move |_, id: String| {
                let store = timer_store(&bridges)?;
                Ok(store.timers.contains_key(&id))
            })?,
        )?;
    }

    globals.set(
        "get_current_year",
        lua.create_function(move |_, ()| Ok(Local::now().year()))?,
    )?;
    globals.set(
        "get_current_month",
        lua.create_function(move |_, ()| Ok(Local::now().month() as i64))?,
    )?;
    globals.set(
        "get_current_day",
        lua.create_function(move |_, ()| Ok(Local::now().day() as i64))?,
    )?;
    globals.set(
        "get_current_hour",
        lua.create_function(move |_, ()| Ok(Local::now().hour() as i64))?,
    )?;
    globals.set(
        "get_current_minute",
        lua.create_function(move |_, ()| Ok(Local::now().minute() as i64))?,
    )?;
    globals.set(
        "get_current_second",
        lua.create_function(move |_, ()| Ok(Local::now().second() as i64))?,
    )?;
    globals.set(
        "timestamp_to_date",
        lua.create_function(move |_, (timestamp, format): (i64, Option<String>)| {
            let format = format.unwrap_or_else(|| {
                "{year}-{month}-{day} {hour}:{minute}:{second}".to_string()
            });
            Ok(format_timestamp(timestamp, &format))
        })?,
    )?;
    globals.set(
        "date_to_timestamp",
        lua.create_function(move |_, date_str: String| Ok(parse_timestamp(&date_str)))?,
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

fn install_timer_mutator<F>(
    lua: &Lua,
    globals: &Table,
    name: &'static str,
    bridges: RuntimeBridges,
    mutator: F,
) -> mlua::Result<()>
where
    F: Fn(&mut TimerEntry) + Clone + Send + 'static,
{
    let apply = mutator.clone();
    globals.set(
        name,
        lua.create_function(move |_, id: String| {
            let mut store = timer_store(&bridges)?;
            let timer = get_timer_mut(&mut store, &id)?;
            apply(timer);
            Ok(())
        })?,
    )?;
    Ok(())
}

fn install_timer_getter<F>(
    lua: &Lua,
    globals: &Table,
    name: &'static str,
    bridges: RuntimeBridges,
    getter: F,
) -> mlua::Result<()>
where
    F: Fn(&TimerEntry) -> Value + Clone + Send + 'static,
{
    let get = getter.clone();
    globals.set(
        name,
        lua.create_function(move |_, id: String| {
            let mut store = timer_store(&bridges)?;
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
        .map_err(|_| mlua::Error::external("timer store poisoned"))
}

fn get_timer_mut<'a>(store: &'a mut TimerStore, id: &str) -> mlua::Result<&'a mut TimerEntry> {
    store
        .timers
        .get_mut(id)
        .ok_or_else(|| mlua::Error::external("timer not found"))
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

fn format_timestamp(timestamp_ms: i64, format: &str) -> String {
    let Some(local_dt) = Local.timestamp_millis_opt(timestamp_ms).single() else {
        return String::new();
    };
    format
        .replace("{year}", &format!("{:04}", local_dt.year()))
        .replace("{month}", &format!("{:02}", local_dt.month()))
        .replace("{day}", &format!("{:02}", local_dt.day()))
        .replace("{hour}", &format!("{:02}", local_dt.hour()))
        .replace("{minute}", &format!("{:02}", local_dt.minute()))
        .replace("{second}", &format!("{:02}", local_dt.second()))
}

fn parse_timestamp(date_str: &str) -> Value {
    let Ok(naive) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") else {
        return Value::Nil;
    };
    let Some(local_dt) = Local.from_local_datetime(&naive).single() else {
        return Value::Nil;
    };
    Value::Integer(local_dt.timestamp_millis())
}
