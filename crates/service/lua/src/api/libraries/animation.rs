//! Arbitrary-value animation tracks using session-owned countdown schedules.

use super::*;
use crate::object_pool::LuaAnimationBinding;
use std::rc::Rc;
use tg_service_animation::AnimationEasing;
use tg_service_time::{
  ScheduledTimerId, ScheduledTimerInfo, ScheduledTimerOptions, TimeService, TimerState,
};

mod parameters;
use parameters::{keyframes_table, parse_keyframes, resolve_easing, selector_copy};

const OPTIONS: &[&str] = &[
  "easing",
  "callback_easing",
  "delay",
  "loop",
  "interval",
  "repeat",
  "direction",
  "fill_mode",
];
const SET_OPTIONS: &[&str] = &[
  "duration",
  "keyframes",
  "easing",
  "callback_easing",
  "delay",
  "loop",
  "interval",
  "repeat",
  "direction",
  "fill_mode",
];
const MAX_ANIMATIONS: usize = 1024;

/// Register the animation API and its immutable string constants.
///
/// # Errors
///
/// Propagate Lua allocation and registration failures.
pub(super) fn animation(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, value) in [
    ("STEP", "step"),
    ("LINEAR", "linear"),
    ("EASE_IN_QUAD", "ease_in_quad"),
    ("EASE_OUT_QUAD", "ease_out_quad"),
    ("EASE_IN_OUT_CUBIC", "ease_in_out_cubic"),
    ("EASE_IN_OUT_REVERSE_CUBIC", "ease_in_out_reverse_cubic"),
    ("NORMAL", "normal"),
    ("REVERSE", "reverse"),
    ("ALTERNATE", "alternate"),
    ("ALTERNATE_REVERSE", "alternate_reverse"),
    ("FORWARDS", "forwards"),
    ("BACKWARDS", "backwards"),
  ] {
    source.raw_set(name, value)?;
  }

  let create_state = state.clone();
  source.raw_set(
    "create",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "animation.create";
      let args = args::positional(lua, method, values, &["duration", "keyframes"], OPTIONS)?;
      let duration = seconds_number(args.required(0, method, "duration")?, method, "duration")?;
      let binding = LuaAnimationBinding {
        timer: ScheduledTimerId(0),
        duration,
        delay: 0.0,
        interval: 0.0,
        looping: false,
        repetitions: None,
        keyframes: Rc::new(parse_keyframes(
          args.required(1, method, "keyframes")?,
          duration,
          method,
        )?),
        easing: Value::String(lua.create_string("step")?),
        callback_easing: Value::String(lua.create_string("step")?),
        direction: "normal".into(),
        fill_mode: "forwards".into(),
      };
      let mut binding = binding_options(lua, binding, args.options(), method)?;
      let options = schedule_options(&binding, method)?;
      with_pool_mut(&create_state, method, |pool| {
        if pool.animations.len() >= MAX_ANIMATIONS {
          return Err(args::message(method, "animation limit of 1024 was reached"));
        }
        let id = pool
          .next_animation_id
          .checked_add(1)
          .ok_or_else(|| args::message(method, "animation identifiers exhausted"))?;
        binding.timer = TimeService::new()
          .create_scheduled_timer(&mut pool.runtime_mut().time, options)
          .map_err(|error| args::message(method, error.to_string()))?;
        pool.next_animation_id = id;
        pool.animations.insert(id, binding);
        Ok(format_id(id))
      })
    })?,
  )?;

  let set_state = state.clone();
  source.raw_set(
    "set",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "animation.set";
      let args = args::positional(lua, method, values, &["id"], SET_OPTIONS)?;
      let id = parse_id(args.required(0, method, "id")?, method)?;
      let original = with_pool(&set_state, method, |pool| {
        pool
          .animations
          .get(&id)
          .cloned()
          .ok_or_else(|| missing_animation(method, id))
      })?;
      let binding = binding_options(lua, original, args.options(), method)?;
      let options = schedule_options(&binding, method)?;
      with_pool_mut(&set_state, method, |pool| {
        let changed = TimeService::new()
          .set_scheduled_timer(&mut pool.runtime_mut().time, binding.timer, options)
          .map_err(|error| args::message(method, error.to_string()))?;
        if changed {
          pool.animations.insert(id, binding);
        }
        Ok(changed)
      })
    })?,
  )?;

  for name in [
    "delete", "play", "pause", "reset", "replay", "exists", "get_info",
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let method = format!("animation.{name}");
        let id = parse_id(args::one(&method, "id", values)?, &method)?;
        with_pool_mut(&state, &method, |pool| {
          let Some(binding) = pool.animations.get(&id).cloned() else {
            return if name == "exists" {
              Ok(Value::Boolean(false))
            } else {
              Err(missing_animation(&method, id))
            };
          };
          let time = TimeService::new();
          let success = match name {
            "exists" => true,
            "delete" => {
              pool.animations.remove(&id);
              time.remove_scheduled_timer(&mut pool.runtime_mut().time, binding.timer)
            }
            "play" => time.start_scheduled_timer(&mut pool.runtime_mut().time, binding.timer),
            "pause" => time.pause_scheduled_timer(&mut pool.runtime_mut().time, binding.timer),
            "reset" => time.reset_scheduled_timer(&mut pool.runtime_mut().time, binding.timer),
            "replay" => {
              time.reset_scheduled_timer(&mut pool.runtime_mut().time, binding.timer)
                && time.start_scheduled_timer(&mut pool.runtime_mut().time, binding.timer)
            }
            "get_info" => return info_table(lua, pool, id, &binding).map(Value::Table),
            _ => unreachable!(),
          };
          Ok(Value::Boolean(success))
        })
      })?,
    )?;
  }
  for name in ["clear", "count", "list"] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let method = format!("animation.{name}");
        args::no_args(&method, values)?;
        with_pool_mut(&state, &method, |pool| match name {
          "count" => Ok(Value::Integer(pool.animations.len() as i64)),
          "clear" => {
            for (_, binding) in std::mem::take(&mut pool.animations) {
              TimeService::new()
                .remove_scheduled_timer(&mut pool.runtime_mut().time, binding.timer);
            }
            Ok(Value::Boolean(true))
          }
          "list" => {
            let output = lua.create_table()?;
            for (index, (&id, binding)) in pool.animations.iter().enumerate() {
              output.raw_set(index + 1, info_table(lua, pool, id, binding)?)?;
            }
            output.raw_set("n", pool.animations.len())?;
            Ok(Value::Table(output))
          }
          _ => unreachable!(),
        })
      })?,
    )?;
  }

  // Lua arithmetic preserves normal numeric and metamethod behavior for continuous easing.
  let interpolate: Function = lua
    .load("return function(a, b, weight) return a + (b - a) * weight end")
    .set_name("animation interpolation")
    .eval()?;
  source.raw_set(
    "sample",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "animation.sample";
      let args = args::positional(lua, method, values, &["id"], &["key"])?;
      let id = parse_id(args.required(0, method, "id")?, method)?;
      let key = args::optional_string(args.options(), method, "key", None)?;
      // Release pool access before running user callbacks, which may mutate or delete animations.
      let snapshot = with_pool(&state, method, |pool| {
        let binding = pool
          .animations
          .get(&id)
          .ok_or_else(|| missing_animation(method, id))?;
        let info = TimeService::new()
          .scheduled_timer_info(&pool.runtime().time, binding.timer)
          .ok_or_else(|| args::message(method, "animation schedule is unavailable"))?;
        Ok((binding.clone(), info))
      })?;
      let (binding, info) = snapshot;
      let time = sample_time(&binding, info);
      if let Some(key) = key {
        return sample_parameter(&binding, &interpolate, &key, time);
      }
      let mut keys = binding.keyframes.keys().peekable();
      if keys.peek().is_none() {
        return Ok(Value::Nil);
      }
      let output = lua.create_table()?;
      for key in keys {
        output.raw_set(key, sample_parameter(&binding, &interpolate, key, time)?)?;
      }
      Ok(Value::Table(output))
    })?,
  )?;
  readonly::proxy(lua, source)
}

fn binding_options(
  lua: &Lua,
  mut binding: LuaAnimationBinding,
  options: &Table,
  method: &str,
) -> mlua::Result<LuaAnimationBinding> {
  for (name, target) in [
    ("duration", &mut binding.duration),
    ("delay", &mut binding.delay),
    ("interval", &mut binding.interval),
  ] {
    let value = options.raw_get::<Value>(name)?;
    if !value.is_nil() {
      *target = seconds_number(value, method, name)?;
    }
  }
  let keyframes = options.raw_get::<Value>("keyframes")?;
  binding.keyframes = Rc::new(if keyframes.is_nil() {
    tg_service_animation::ParameterTrack::new(
      seconds(binding.duration, method, "duration")?,
      binding.keyframes.frames().to_vec(),
    )
    .map_err(|error| args::message(method, error.to_string()))?
  } else {
    parse_keyframes(keyframes, binding.duration, method)?
  });
  for (name, target, segments) in [
    ("easing", &mut binding.easing, true),
    ("callback_easing", &mut binding.callback_easing, false),
  ] {
    let value = options.raw_get::<Value>(name)?;
    if !value.is_nil() {
      *target = selector_copy(lua, value, method, segments)?;
    }
  }
  binding.looping = args::optional_bool(options, method, "loop", binding.looping)?;
  match options.raw_get::<Value>("repeat")? {
    Value::Nil => {}
    Value::Number(value) if value == f64::INFINITY => binding.repetitions = None,
    value => {
      let value = args::integer(value, method, "repeat")?;
      binding.repetitions = Some(
        u32::try_from(value)
          .ok()
          .filter(|value| *value > 0)
          .ok_or_else(|| {
            args::message(method, "repeat must be 1..=4294967295 or positive infinity")
          })?,
      );
    }
  }
  for (name, target, choices) in [
    (
      "direction",
      &mut binding.direction,
      &["normal", "reverse", "alternate", "alternate_reverse"][..],
    ),
    (
      "fill_mode",
      &mut binding.fill_mode,
      &["forwards", "backwards"][..],
    ),
  ] {
    if let Some(value) = args::optional_string(options, method, name, None)? {
      if !choices.contains(&value.as_str()) {
        return Err(args::message(method, format!("invalid {name}")));
      }
      *target = value;
    }
  }
  Ok(binding)
}

fn seconds_number(value: Value, method: &str, name: &str) -> mlua::Result<f64> {
  let value = args::number(value, method, name)?;
  seconds(value, method, name)?;
  Ok(value)
}

fn seconds(value: f64, method: &str, name: &str) -> mlua::Result<Duration> {
  Duration::try_from_secs_f64(value).map_err(|_| {
    args::message(
      method,
      format!("{name} must be finite, non-negative and fit the duration range"),
    )
  })
}

fn schedule_options(
  binding: &LuaAnimationBinding,
  method: &str,
) -> mlua::Result<ScheduledTimerOptions> {
  Ok(ScheduledTimerOptions {
    duration: seconds(binding.duration, method, "duration")?,
    delay: seconds(binding.delay, method, "delay")?,
    gap: seconds(binding.interval, method, "interval")?,
    repetitions: if binding.looping {
      binding.repetitions
    } else {
      Some(1)
    },
  })
}

fn sample_time(binding: &LuaAnimationBinding, info: ScheduledTimerInfo) -> Duration {
  let duration = info.options.duration;
  let (cycle, elapsed) = if info.state == TimerState::Finished {
    (info.executed_count.saturating_sub(1), duration)
  } else {
    let wait = if info.executed_count == 0 {
      info.options.delay
    } else {
      info.options.gap
    };
    if info.elapsed < wait && info.executed_count > 0 && binding.fill_mode == "forwards" {
      (info.executed_count - 1, duration)
    } else {
      (
        info.executed_count,
        info.elapsed.saturating_sub(wait).min(duration),
      )
    }
  };
  let reverse = match binding.direction.as_str() {
    "reverse" => true,
    "alternate" => cycle % 2 == 1,
    "alternate_reverse" => cycle % 2 == 0,
    _ => false,
  };
  if reverse {
    duration.saturating_sub(elapsed)
  } else {
    elapsed
  }
}

fn sample_parameter(
  binding: &LuaAnimationBinding,
  interpolate: &Function,
  key: &str,
  time: Duration,
) -> mlua::Result<Value> {
  let Some(segment) = binding.keyframes.segment(key, time) else {
    return Ok(Value::Nil);
  };
  let current = time.clamp(segment.start_time, segment.end_time);
  let easing = resolve_easing(
    &binding.easing,
    &binding.callback_easing,
    key,
    segment.index,
  )?;
  if let Value::Function(function) = easing {
    return function.call((
      current.as_secs_f64(),
      segment.start_time.as_secs_f64(),
      segment.end_time.as_secs_f64(),
      segment.start_value.clone(),
      segment.end_value.clone(),
    ));
  }
  let name = if easing.is_nil() {
    "step".into()
  } else {
    args::string(easing, "animation.sample", "easing")?
  };
  if name == "step" {
    return Ok(
      if current < segment.end_time {
        segment.start_value
      } else {
        segment.end_value
      }
      .clone(),
    );
  }
  let progress = if segment.end_time == segment.start_time {
    1.0
  } else {
    (current - segment.start_time).as_secs_f64()
      / (segment.end_time - segment.start_time).as_secs_f64()
  };
  let easing = match name.as_str() {
    "linear" => AnimationEasing::Linear,
    "ease_in_quad" => AnimationEasing::InQuad,
    "ease_out_quad" => AnimationEasing::OutQuad,
    "ease_in_out_cubic" => AnimationEasing::InOutCubic,
    "ease_in_out_reverse_cubic" => AnimationEasing::OutInCubic,
    _ => unreachable!("easing selectors are validated before storage"),
  };
  interpolate.call((
    segment.start_value.clone(),
    segment.end_value.clone(),
    easing.sample(progress),
  ))
}

fn info_table(
  lua: &Lua,
  pool: &crate::LuaObjectPool,
  id: u64,
  binding: &LuaAnimationBinding,
) -> mlua::Result<Table> {
  let info = TimeService::new()
    .scheduled_timer_info(&pool.runtime().time, binding.timer)
    .ok_or_else(|| args::message("animation.get_info", "animation schedule is unavailable"))?;
  let output = lua.create_table()?;
  output.raw_set("id", format_id(id))?;
  output.raw_set("duration", binding.duration)?;
  output.raw_set("delay", binding.delay)?;
  output.raw_set("interval", binding.interval)?;
  output.raw_set("loop", binding.looping)?;
  output.raw_set(
    "repeat",
    binding
      .repetitions
      .map_or(Value::Number(f64::INFINITY), |count| {
        Value::Integer(i64::from(count))
      }),
  )?;
  output.raw_set("keyframes", keyframes_table(lua, &binding.keyframes)?)?;
  output.raw_set(
    "easing",
    selector_copy(lua, binding.easing.clone(), "animation.get_info", true)?,
  )?;
  output.raw_set(
    "callback_easing",
    selector_copy(
      lua,
      binding.callback_easing.clone(),
      "animation.get_info",
      false,
    )?,
  )?;
  output.raw_set("direction", binding.direction.as_str())?;
  output.raw_set(
    "fill_mode",
    if binding.looping {
      &binding.fill_mode
    } else {
      "forwards"
    },
  )?;
  output.raw_set(
    "state",
    match info.state {
      TimerState::Idle => "idle",
      TimerState::Delay => "delay",
      TimerState::Running => "running",
      TimerState::Interval => "interval",
      TimerState::Paused => "paused",
      TimerState::Finished => "finished",
      TimerState::Stopped => "stopped",
    },
  )?;
  output.raw_set("elapsed", info.elapsed.as_secs_f64())?;
  output.raw_set("remaining", info.remaining.as_secs_f64())?;
  output.raw_set("executed_count", info.executed_count)?;
  Ok(output)
}

fn parse_id(value: Value, method: &str) -> mlua::Result<u64> {
  args::string(value, method, "id")?
    .strip_prefix("animation_")
    .and_then(|value| value.parse().ok())
    .filter(|value| *value > 0)
    .ok_or_else(|| args::message(method, "invalid animation ID"))
}

fn format_id(id: u64) -> String {
  format!("animation_{id:03}")
}

fn missing_animation(method: &str, id: u64) -> mlua::Error {
  args::message(
    method,
    format!("animation '{}' does not exist", format_id(id)),
  )
}
