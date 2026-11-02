//! Validate parameter keyframes and resolve field-specific easing fallbacks.

use super::*;
use std::collections::{BTreeMap, BTreeSet};
use tg_service_animation::{ParameterKeyframe, ParameterTrack};

pub(super) fn parse_track(
  value: Value,
  duration: f64,
  method: &str,
) -> mlua::Result<ParameterTrack<Value>> {
  let rows = args::array_values(value, method, "track")?;
  let mut frames = Vec::with_capacity(rows.len());
  let mut parameters = BTreeSet::new();
  let mut entries = 0;
  for row in rows {
    let row = plain_table(row, method, "track entry")?;
    let timestamp = seconds_number(
      args::required(&row, method, "timestamp")?,
      method,
      "timestamp",
    )?;
    if timestamp > duration {
      return Err(args::message(method, "timestamp exceeds duration"));
    }
    let keys = plain_table(args::required(&row, method, "key")?, method, "key")?;
    let mut values = BTreeMap::new();
    for pair in keys.pairs::<Value, Value>() {
      entries += 1;
      if entries > args::MAX_API_TABLE_ENTRIES {
        return Err(args::message(method, "track exceeds 16384 values"));
      }
      let (key, value) = pair?;
      let key = args::string(key, method, "parameter name")?;
      parameters.insert(key.clone());
      values.insert(key, value);
    }
    frames.push(ParameterKeyframe {
      timestamp: seconds(timestamp, method, "timestamp")?,
      values,
    });
  }
  // Boundary padding must stay bounded even when each frame introduces a different parameter.
  if frames.len().saturating_mul(parameters.len()) > args::MAX_API_TABLE_ENTRIES {
    return Err(args::message(
      method,
      "normalized track exceeds 16384 parameter positions",
    ));
  }
  ParameterTrack::new(seconds(duration, method, "duration")?, frames)
    .map_err(|error| args::message(method, error.to_string()))
}

pub(super) fn track_table(lua: &Lua, track: &ParameterTrack<Value>) -> mlua::Result<Table> {
  let output = lua.create_table()?;
  for (index, frame) in track.frames().iter().enumerate() {
    let row = lua.create_table()?;
    row.raw_set("timestamp", frame.timestamp.as_secs_f64())?;
    let keys = lua.create_table()?;
    for (key, value) in &frame.values {
      keys.raw_set(key.as_str(), value.clone())?;
    }
    row.raw_set("key", keys)?;
    output.raw_set(index + 1, row)?;
  }
  Ok(output)
}

pub(super) fn selector_copy(
  lua: &Lua,
  value: Value,
  method: &str,
  segments: bool,
) -> mlua::Result<Value> {
  if !matches!(value, Value::Table(_)) {
    return validate_easing(value, method);
  }
  let table = plain_table(value, method, "easing selector")?;
  let output = lua.create_table()?;
  let mut entries = 0;
  for pair in table.pairs::<Value, Value>() {
    let (key, value) = pair?;
    entries += 1;
    let key = args::string(key, method, "easing parameter name")?;
    let value = if segments && matches!(value, Value::Table(_)) {
      let input = plain_table(value, method, "segment easing")?;
      let selected = lua.create_table()?;
      for pair in input.pairs::<Value, Value>() {
        entries += 1;
        if entries > args::MAX_API_TABLE_ENTRIES {
          return Err(args::message(method, "easing exceeds 16384 entries"));
        }
        let (index, value) = pair?;
        let index = args::integer(index, method, "segment index")?;
        if index < 1 {
          return Err(args::message(method, "segment index must be positive"));
        }
        selected.raw_set(index, validate_easing(value, method)?)?;
      }
      Value::Table(selected)
    } else {
      validate_easing(value, method)?
    };
    if entries > args::MAX_API_TABLE_ENTRIES {
      return Err(args::message(method, "easing exceeds 16384 entries"));
    }
    output.raw_set(key, value)?;
  }
  Ok(Value::Table(output))
}

fn validate_easing(value: Value, method: &str) -> mlua::Result<Value> {
  if matches!(value, Value::Function(_)) {
    return Ok(value);
  }
  let name = args::string(value.clone(), method, "easing")?;
  if ![
    "step",
    "linear",
    "ease_in_quad",
    "ease_out_quad",
    "ease_in_out_cubic",
    "ease_in_out_reverse_cubic",
  ]
  .contains(&name.as_str())
  {
    return Err(args::message(method, "unknown easing constant"));
  }
  Ok(value)
}

pub(super) fn resolve_easing(
  easing: &Value,
  fallback: &Value,
  key: &str,
  index: usize,
) -> mlua::Result<Value> {
  let selected = if let Value::Table(table) = easing {
    match table.raw_get::<Value>(key)? {
      Value::Table(segments) => segments.raw_get::<Value>(index)?,
      value => value,
    }
  } else {
    easing.clone()
  };
  if !selected.is_nil() {
    return Ok(selected);
  }
  let fallback = if let Value::Table(table) = fallback {
    table.raw_get::<Value>(key)?
  } else {
    fallback.clone()
  };
  Ok(fallback)
}

fn plain_table(value: Value, method: &str, name: &str) -> mlua::Result<Table> {
  match value {
    Value::Table(table) if table.metatable().is_none() => Ok(table),
    value => Err(args::invalid(
      method,
      name,
      "table without metatable",
      &value,
    )),
  }
}
