//! Session-owned countdown timer bindings with strict options and bounded creation.

use super::*;
use crate::object_pool::LuaTimerBinding;
use tg_service_time::{ScheduledTimerId, ScheduledTimerOptions, TimeService, TimerState};

const MAX_TIMERS: usize = 1024;
const OPTIONS: &[&str] = &["delay", "loop", "interval", "repeat", "callback", "tip"];
const SET_OPTIONS: &[&str] = &[
  "duration", "delay", "loop", "interval", "repeat", "callback", "tip",
];

/// Build the timer library in the supplied VM and session context.
///
/// # Errors
///
/// Propagate Lua allocation and registration failures.
pub(super) fn timer(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  let create_state = state.clone();
  source.raw_set(
    "create",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "timer.create";
      let parameters = args::positional(lua, method, values, &["duration"], OPTIONS)?;
      let duration = number(
        parameters.required(0, method, "duration")?,
        method,
        "duration",
      )?;
      let binding = binding_options(
        parameters.options(),
        method,
        LuaTimerBinding {
          duration,
          delay: 0.0,
          interval: 0.0,
          looping: false,
          repetitions: None,
          callback: None,
          tip: None,
        },
      )?;
      let options = schedule_options(&binding, method)?;
      with_pool_mut(&create_state, method, |pool| {
        if pool.timers.len() >= MAX_TIMERS {
          return Err(args::message(method, "timer limit of 1024 was reached"));
        }
        let id = TimeService::new()
          .create_scheduled_timer(&mut pool.runtime_mut().time, options)
          .map_err(|error| args::message(method, error.to_string()))?;
        pool.timers.insert(id, binding);
        Ok(format_id(id))
      })
    })?,
  )?;

  let set_state = state.clone();
  source.raw_set(
    "set",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "timer.set";
      let parameters = args::positional(lua, method, values, &["id"], SET_OPTIONS)?;
      let id = parse_id(
        args::string(parameters.required(0, method, "id")?, method, "id")?,
        method,
      )?;
      with_pool_mut(&set_state, method, |pool| {
        let Some(original) = pool.timers.get(&id) else {
          return Ok(false);
        };
        let binding = binding_options(parameters.options(), method, original.clone())?;
        let options = schedule_options(&binding, method)?;
        let changed = TimeService::new()
          .set_scheduled_timer(&mut pool.runtime_mut().time, id, options)
          .map_err(|error| args::message(method, error.to_string()))?;
        if changed {
          pool.timers.insert(id, binding);
        }
        Ok(changed)
      })
    })?,
  )?;

  for (name, method) in [
    ("delete", "timer.delete"),
    ("start", "timer.start"),
    ("pause", "timer.pause"),
    ("reset", "timer.reset"),
    ("restart", "timer.restart"),
    ("exists", "timer.exists"),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let id = id_argument(values, method)?;
        with_pool_mut(&state, method, |pool| {
          if !pool.timers.contains_key(&id) {
            return Ok(false);
          }
          let service = TimeService::new();
          let result = match name {
            "exists" => true,
            "delete" => {
              pool.timers.remove(&id);
              service.remove_scheduled_timer(&mut pool.runtime_mut().time, id)
            }
            "start" => service.start_scheduled_timer(&mut pool.runtime_mut().time, id),
            "pause" => service.pause_scheduled_timer(&mut pool.runtime_mut().time, id),
            "reset" => service.reset_scheduled_timer(&mut pool.runtime_mut().time, id),
            "restart" => {
              service.reset_scheduled_timer(&mut pool.runtime_mut().time, id)
                && service.start_scheduled_timer(&mut pool.runtime_mut().time, id)
            }
            _ => unreachable!(),
          };
          Ok(result)
        })
      })?,
    )?;
  }

  let info_state = state.clone();
  source.raw_set(
    "get_info",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "timer.get_info";
      let id = id_argument(values, method)?;
      with_pool(&info_state, method, |pool| {
        info_table(lua, pool, id).map(|table| table.map_or(Value::Nil, Value::Table))
      })
    })?,
  )?;

  let list_state = state.clone();
  source.raw_set(
    "list",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "timer.list";
      args::no_args(method, values)?;
      with_pool(&list_state, method, |pool| {
        let result = lua.create_table()?;
        for (index, &id) in pool.timers.keys().enumerate() {
          if let Some(info) = info_table(lua, pool, id)? {
            result.raw_set(index + 1, info)?;
          }
        }
        result.raw_set("n", pool.timers.len())?;
        Ok(result)
      })
    })?,
  )?;

  let count_state = state.clone();
  source.raw_set(
    "count",
    lua.create_function(move |_, values: MultiValue| {
      args::no_args("timer.count", values)?;
      with_pool(&count_state, "timer.count", |pool| Ok(pool.timers.len()))
    })?,
  )?;
  source.raw_set(
    "clear",
    lua.create_function(move |_, values: MultiValue| {
      args::no_args("timer.clear", values)?;
      with_pool_mut(&state, "timer.clear", |pool| {
        let ids = pool.timers.keys().copied().collect::<Vec<_>>();
        pool.timers.clear();
        for id in ids {
          TimeService::new().remove_scheduled_timer(&mut pool.runtime_mut().time, id);
        }
        Ok(true)
      })
    })?,
  )?;
  readonly::proxy(lua, source)
}

fn binding_options(
  table: &Table,
  method: &str,
  mut binding: LuaTimerBinding,
) -> mlua::Result<LuaTimerBinding> {
  for (name, target) in [
    ("duration", &mut binding.duration),
    ("delay", &mut binding.delay),
    ("interval", &mut binding.interval),
  ] {
    let value = table.raw_get::<Value>(name)?;
    if !matches!(value, Value::Nil) {
      *target = number(value, method, name)?;
    }
  }
  binding.looping = args::optional_bool(table, method, "loop", binding.looping)?;
  match table.raw_get::<Value>("repeat")? {
    Value::Nil => {}
    Value::Boolean(false) => binding.repetitions = None,
    value => {
      let count = args::integer(value, method, "repeat")?;
      binding.repetitions = Some(
        u32::try_from(count)
          .ok()
          .filter(|count| *count > 0)
          .ok_or_else(|| {
            args::message(
              method,
              "repeat must be an integer from 1 to 4294967295, or false",
            )
          })?,
      );
    }
  }
  match table.raw_get::<Value>("callback")? {
    Value::Nil => {}
    Value::Boolean(false) => binding.callback = None,
    Value::Function(function) => binding.callback = Some(function),
    _ => {
      return Err(args::message(
        method,
        "callback must be a function or false",
      ));
    }
  }
  match table.raw_get::<Value>("tip")? {
    Value::Nil => {}
    Value::Boolean(false) => binding.tip = None,
    value => binding.tip = Some(args::string(value, method, "tip")?),
  }
  Ok(binding)
}

fn number(value: Value, method: &str, name: &str) -> mlua::Result<f64> {
  let value = args::number(value, method, name)?;
  if !value.is_finite() {
    return Err(args::message(method, format!("{name} must be finite")));
  }
  Ok(value)
}

fn seconds(value: f64, method: &str, name: &str) -> mlua::Result<Duration> {
  Duration::try_from_secs_f64(value).map_err(|_| {
    args::message(
      method,
      format!("{name} must be non-negative and fit the duration range"),
    )
  })
}

fn schedule_options(
  binding: &LuaTimerBinding,
  method: &str,
) -> mlua::Result<ScheduledTimerOptions> {
  let duration = seconds(binding.duration, method, "duration")?;
  let delay = seconds(binding.delay, method, "delay")?;
  let gap = seconds(binding.delay + binding.interval, method, "delay + interval")?;
  Ok(ScheduledTimerOptions {
    duration,
    delay,
    gap,
    repetitions: if binding.looping {
      binding.repetitions
    } else {
      Some(1)
    },
  })
}

fn info_table(
  lua: &Lua,
  pool: &crate::LuaObjectPool,
  id: ScheduledTimerId,
) -> mlua::Result<Option<Table>> {
  let Some(binding) = pool.timers.get(&id) else {
    return Ok(None);
  };
  let Some(info) = TimeService::new().scheduled_timer_info(&pool.runtime().time, id) else {
    return Ok(None);
  };
  let result = lua.create_table()?;
  result.raw_set("id", format_id(id))?;
  result.raw_set("duration", binding.duration)?;
  result.raw_set("delay", binding.delay)?;
  result.raw_set("interval", binding.interval)?;
  result.raw_set("loop", binding.looping)?;
  result.raw_set("repeat", binding.repetitions)?;
  result.raw_set("callback", binding.callback.clone())?;
  result.raw_set("tip", binding.tip.as_deref())?;
  result.raw_set(
    "state",
    match info.state {
      TimerState::Idle => "idle",
      TimerState::Running => "running",
      TimerState::Paused => "paused",
      TimerState::Finished => "finished",
      TimerState::Stopped => "stopped",
    },
  )?;
  result.raw_set("elapsed", info.elapsed.as_secs_f64())?;
  result.raw_set("remaining", info.remaining.as_secs_f64())?;
  result.raw_set("executed_count", info.executed_count)?;
  Ok(Some(result))
}

fn id_argument(values: MultiValue, method: &str) -> mlua::Result<ScheduledTimerId> {
  parse_id(
    args::string(args::one(method, "id", values)?, method, "id")?,
    method,
  )
}

fn parse_id(value: String, method: &str) -> mlua::Result<ScheduledTimerId> {
  value
    .strip_prefix("timer_")
    .and_then(|value| value.parse::<u64>().ok())
    .filter(|value| *value > 0)
    .map(ScheduledTimerId)
    .ok_or_else(|| args::message(method, "invalid timer ID"))
}

fn format_id(id: ScheduledTimerId) -> String {
  format!("timer_{:03}", id.0)
}
