use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mlua::{Lua, MultiValue, Table, Value};

use super::*;
use crate::host_engine::services::{
  RandomConfiguration, RandomConfiguredRange, RandomGeneratedValue, RandomGeneratorId, RandomSeed,
  RandomService,
};

const MAX_GENERATORS: usize = 4096;
static AUTO_SEED_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub(super) fn random(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  source.raw_set("INT", "int")?;
  source.raw_set("FLOAT", "float")?;

  install_direct(lua, &source, state.clone(), true)?;
  install_direct(lua, &source, state.clone(), false)?;
  install_lifecycle(lua, &source, state.clone())?;
  install_mutations(lua, &source, state.clone())?;
  install_queries(lua, &source, state)?;
  readonly::proxy(lua, source)
}

fn install_direct(
  lua: &Lua,
  source: &Table,
  state: SharedApiState,
  integer: bool,
) -> mlua::Result<()> {
  let name = if integer { "randint" } else { "randfloat" };
  let method = if integer {
    "random.randint"
  } else {
    "random.randfloat"
  };
  source.raw_set(
    name,
    lua.create_function(move |_lua, values: MultiValue| {
      let table = args::named(method, values, &["min", "max"])?;
      if integer {
        let min = args::optional_integer(&table, method, "min", Some(i32::MIN.into()))?.unwrap();
        let max = args::optional_integer(&table, method, "max", Some(i32::MAX.into()))?.unwrap();
        if min > max {
          return Err(args::message(
            method,
            "min must be less than or equal to max",
          ));
        }
        with_direct_generator(&state, method, |service, pool, id| {
          service
            .int_range_inclusive(pool, id, min, max)
            .map(Value::Integer)
            .ok_or_else(|| args::message(method, "random generator is unavailable"))
        })
      } else {
        let min = optional_finite_number(&table, method, "min", 0.0)?;
        let max = optional_finite_number(&table, method, "max", 1.0)?;
        if min > max {
          return Err(args::message(
            method,
            "min must be less than or equal to max",
          ));
        }
        with_direct_generator(&state, method, |service, pool, id| {
          service
            .float_range_inclusive(pool, id, min, max)
            .map(Value::Number)
            .ok_or_else(|| args::message(method, "random generator is unavailable"))
        })
      }
    })?,
  )
}

fn install_lifecycle(lua: &Lua, source: &Table, state: SharedApiState) -> mlua::Result<()> {
  let create_state = state.clone();
  source.raw_set(
    "create",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "random.create";
      let table = args::named(method, values, &["type", "min", "max", "seed", "step"])?;
      let configuration = configuration_from_create(&table, method)?;
      with_pool_mut(&create_state, method, |pool| {
        let service = RandomService::new();
        if service.configured_ids(pool.runtime()).len() >= MAX_GENERATORS {
          return Err(args::message(method, "generator limit of 4096 was reached"));
        }
        let id = service.create_configured(pool.runtime_mut(), configuration);
        Ok(Value::String(lua.create_string(format_id(id))?))
      })
    })?,
  )?;

  let delete_state = state.clone();
  source.raw_set(
    "delete",
    lua.create_function(move |_, values: MultiValue| {
      let method = "random.delete";
      let id = id_argument(values, method)?;
      with_pool_mut(&delete_state, method, |pool| {
        Ok(RandomService::new().remove(pool.runtime_mut(), id))
      })
    })?,
  )?;

  let clear_state = state.clone();
  source.raw_set(
    "clear",
    lua.create_function(move |_, values: MultiValue| {
      let method = "random.clear";
      args::no_args(method, values)?;
      with_pool_mut(&clear_state, method, |pool| {
        RandomService::new().clear_configured(pool.runtime_mut());
        Ok(true)
      })
    })?,
  )?;

  let list_state = state.clone();
  source.raw_set(
    "list",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "random.list";
      args::no_args(method, values)?;
      with_pool(&list_state, method, |pool| {
        let result = lua.create_table()?;
        let service = RandomService::new();
        let ids = service.configured_ids(pool.runtime());
        for (index, id) in ids.iter().copied().enumerate() {
          let configuration = service.configuration(pool.runtime(), id).unwrap();
          result.raw_set(index + 1, configuration_table(lua, id, configuration)?)?;
        }
        result.raw_set("n", ids.len())?;
        Ok(result)
      })
    })?,
  )?;

  let count_state = state.clone();
  source.raw_set(
    "count",
    lua.create_function(move |_, values: MultiValue| {
      let method = "random.count";
      args::no_args(method, values)?;
      with_pool(&count_state, method, |pool| {
        Ok(RandomService::new().configured_ids(pool.runtime()).len())
      })
    })?,
  )?;

  source.raw_set(
    "generate",
    lua.create_function(move |_, values: MultiValue| {
      let method = "random.generate";
      let id = id_argument(values, method)?;
      with_pool_mut(&state, method, |pool| {
        let service = RandomService::new();
        if service
          .configuration(pool.runtime(), id)
          .is_some_and(|configuration| configuration.step >= i64::MAX as u64)
        {
          return Err(args::message(method, "generator step is exhausted"));
        }
        Ok(match service.generate_configured(pool.runtime_mut(), id) {
          Some(RandomGeneratedValue::Integer(value)) => Value::Integer(value),
          Some(RandomGeneratedValue::Float(value)) => Value::Number(value),
          None => Value::Nil,
        })
      })
    })?,
  )
}

fn install_mutations(lua: &Lua, source: &Table, state: SharedApiState) -> mlua::Result<()> {
  for (name, allowed) in [
    ("set", &["id", "type", "min", "max", "seed", "step"][..]),
    ("set_type", &["id", "type"][..]),
    ("set_range", &["id", "min", "max"][..]),
    ("set_seed", &["id", "seed"][..]),
    ("set_step", &["id", "step"][..]),
  ] {
    let method: &'static str = match name {
      "set" => "random.set",
      "set_type" => "random.set_type",
      "set_range" => "random.set_range",
      "set_seed" => "random.set_seed",
      _ => "random.set_step",
    };
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_lua, values: MultiValue| {
        let table = args::named(method, values, allowed)?;
        if name == "set_range" {
          args::required(&table, method, "min")?;
          args::required(&table, method, "max")?;
        } else if name == "set_step" {
          args::required(&table, method, "step")?;
        }
        let id = parse_id(args::string(
          args::required(&table, method, "id")?,
          method,
          "id",
        )?)
        .ok_or_else(|| args::message(method, "invalid generator ID"))?;
        with_pool_mut(&state, method, |pool| {
          let service = RandomService::new();
          let Some(current) = service.configuration(pool.runtime(), id) else {
            return Ok(false);
          };
          let updated = update_configuration(current, &table, method)?;
          Ok(service.set_configuration(pool.runtime_mut(), id, updated))
        })
      })?,
    )?;
  }
  Ok(())
}

fn install_queries(lua: &Lua, source: &Table, state: SharedApiState) -> mlua::Result<()> {
  for name in ["get_type", "get_seed", "get_step", "exists"] {
    let method: &'static str = match name {
      "get_type" => "random.get_type",
      "get_seed" => "random.get_seed",
      "get_step" => "random.get_step",
      _ => "random.exists",
    };
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let id = id_argument(values, method)?;
        with_pool(&state, method, |pool| {
          let configuration = RandomService::new().configuration(pool.runtime(), id);
          Ok(match name {
            "get_type" => match configuration {
              Some(configuration) => {
                Value::String(lua.create_string(match configuration.range {
                  RandomConfiguredRange::Integer { .. } => "int",
                  RandomConfiguredRange::Float { .. } => "float",
                })?)
              }
              None => Value::Nil,
            },
            "get_seed" => configuration.map_or(Value::Nil, |value| Value::Integer(value.seed)),
            "get_step" => configuration
              .and_then(|value| i64::try_from(value.step).ok())
              .map_or(Value::Nil, Value::Integer),
            _ => Value::Boolean(configuration.is_some()),
          })
        })
      })?,
    )?;
  }

  let range_state = state.clone();
  source.raw_set(
    "get_range",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "random.get_range";
      let id = id_argument(values, method)?;
      with_pool(&range_state, method, |pool| {
        match RandomService::new().configuration(pool.runtime(), id) {
          Some(RandomConfiguration {
            range: RandomConfiguredRange::Integer { min, max },
            ..
          }) => {
            let result = lua.create_table()?;
            result.raw_set("min", min)?;
            result.raw_set("max", max)?;
            Ok(Value::Table(result))
          }
          Some(RandomConfiguration {
            range: RandomConfiguredRange::Float { min, max },
            ..
          }) => {
            let result = lua.create_table()?;
            result.raw_set("min", min)?;
            result.raw_set("max", max)?;
            Ok(Value::Table(result))
          }
          None => Ok(Value::Nil),
        }
      })
    })?,
  )?;

  source.raw_set(
    "get_info",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "random.get_info";
      let id = id_argument(values, method)?;
      with_pool(&state, method, |pool| {
        let Some(configuration) = RandomService::new().configuration(pool.runtime(), id) else {
          return Ok(Value::Nil);
        };
        Ok(Value::Table(configuration_table(lua, id, configuration)?))
      })
    })?,
  )
}

fn configuration_from_create(table: &Table, method: &str) -> mlua::Result<RandomConfiguration> {
  let kind = args::optional_string(table, method, "type", Some("int"))?.unwrap();
  let range = parse_optional_range(kind.as_str(), table, method)?;
  let seed = match table.get::<Value>("seed")? {
    Value::Nil => auto_seed() as i64,
    value => args::integer(value, method, "seed")?,
  };
  let step = non_negative_step(
    args::optional_integer(table, method, "step", Some(0))?.unwrap_or(0),
    method,
  )?;
  Ok(RandomConfiguration { range, seed, step })
}

fn parse_optional_range(
  kind: &str,
  table: &Table,
  method: &str,
) -> mlua::Result<RandomConfiguredRange> {
  let range = match kind {
    "int" => RandomConfiguredRange::Integer {
      min: args::optional_integer(table, method, "min", Some(i32::MIN.into()))?.unwrap(),
      max: args::optional_integer(table, method, "max", Some(i32::MAX.into()))?.unwrap(),
    },
    "float" => RandomConfiguredRange::Float {
      min: optional_finite_number(table, method, "min", 0.0)?,
      max: optional_finite_number(table, method, "max", 1.0)?,
    },
    _ => {
      return Err(args::message(
        method,
        "type must be random.INT or random.FLOAT",
      ));
    }
  };
  match range {
    RandomConfiguredRange::Integer { min, max } if min > max => Err(args::message(
      method,
      "min must be less than or equal to max",
    )),
    RandomConfiguredRange::Float { min, max } if min > max => Err(args::message(
      method,
      "min must be less than or equal to max",
    )),
    _ => Ok(range),
  }
}

fn update_configuration(
  current: RandomConfiguration,
  table: &Table,
  method: &str,
) -> mlua::Result<RandomConfiguration> {
  let requested_kind = args::optional_string(table, method, "type", None)?;
  if requested_kind
    .as_deref()
    .is_some_and(|kind| kind != "int" && kind != "float")
  {
    return Err(args::message(
      method,
      "type must be random.INT or random.FLOAT",
    ));
  }
  let target_kind = requested_kind.as_deref().unwrap_or(match current.range {
    RandomConfiguredRange::Integer { .. } => "int",
    RandomConfiguredRange::Float { .. } => "float",
  });
  let min_value = table.get::<Value>("min")?;
  let max_value = table.get::<Value>("max")?;
  let range = match target_kind {
    "int" => {
      let (old_min, old_max) = integer_bounds(current.range, method)?;
      let min = if matches!(min_value, Value::Nil) {
        old_min
      } else {
        args::integer(min_value, method, "min")?
      };
      let max = if matches!(max_value, Value::Nil) {
        old_max
      } else {
        args::integer(max_value, method, "max")?
      };
      if min > max {
        return Err(args::message(
          method,
          "min must be less than or equal to max",
        ));
      }
      RandomConfiguredRange::Integer { min, max }
    }
    "float" => {
      let (old_min, old_max) = float_bounds(current.range);
      let min = if matches!(min_value, Value::Nil) {
        old_min
      } else {
        finite_number(min_value, method, "min")?
      };
      let max = if matches!(max_value, Value::Nil) {
        old_max
      } else {
        finite_number(max_value, method, "max")?
      };
      if min > max {
        return Err(args::message(
          method,
          "min must be less than or equal to max",
        ));
      }
      RandomConfiguredRange::Float { min, max }
    }
    _ => {
      return Err(args::message(
        method,
        "type must be random.INT or random.FLOAT",
      ));
    }
  };
  let seed = match table.get::<Value>("seed")? {
    Value::Nil => current.seed,
    value => args::integer(value, method, "seed")?,
  };
  let step = match table.get::<Value>("step")? {
    Value::Nil => current.step,
    value => non_negative_step(args::integer(value, method, "step")?, method)?,
  };
  Ok(RandomConfiguration { range, seed, step })
}

fn integer_bounds(range: RandomConfiguredRange, method: &str) -> mlua::Result<(i64, i64)> {
  match range {
    RandomConfiguredRange::Integer { min, max } => Ok((min, max)),
    RandomConfiguredRange::Float { min, max }
      if min.fract() == 0.0
        && max.fract() == 0.0
        && min >= -9_223_372_036_854_775_808.0
        && max < 9_223_372_036_854_775_808.0 =>
    {
      Ok((min as i64, max as i64))
    }
    _ => Err(args::message(
      method,
      "floating range cannot be converted to an integer range",
    )),
  }
}

fn float_bounds(range: RandomConfiguredRange) -> (f64, f64) {
  match range {
    RandomConfiguredRange::Integer { min, max } => (min as f64, max as f64),
    RandomConfiguredRange::Float { min, max } => (min, max),
  }
}

fn finite_number(value: Value, method: &str, name: &str) -> mlua::Result<f64> {
  let value = args::number(value, method, name)?;
  if value.is_finite() {
    Ok(value)
  } else {
    Err(args::message(method, format!("{name} must be finite")))
  }
}

fn optional_finite_number(
  table: &Table,
  method: &str,
  name: &str,
  default: f64,
) -> mlua::Result<f64> {
  match table.get::<Value>(name)? {
    Value::Nil => Ok(default),
    value => finite_number(value, method, name),
  }
}

fn configuration_table(
  lua: &Lua,
  id: RandomGeneratorId,
  configuration: RandomConfiguration,
) -> mlua::Result<Table> {
  let info = lua.create_table()?;
  info.raw_set("id", format_id(id))?;
  match configuration.range {
    RandomConfiguredRange::Integer { min, max } => {
      info.raw_set("type", "int")?;
      info.raw_set("min", min)?;
      info.raw_set("max", max)?;
    }
    RandomConfiguredRange::Float { min, max } => {
      info.raw_set("type", "float")?;
      info.raw_set("min", min)?;
      info.raw_set("max", max)?;
    }
  }
  info.raw_set("seed", configuration.seed)?;
  info.raw_set("step", configuration.step)?;
  Ok(info)
}

fn non_negative_step(value: i64, method: &str) -> mlua::Result<u64> {
  u64::try_from(value).map_err(|_| args::message(method, "step must be non-negative"))
}

fn id_argument(values: MultiValue, method: &str) -> mlua::Result<RandomGeneratorId> {
  let id = args::string(args::one(method, "id", values)?, method, "id")?;
  parse_id(id).ok_or_else(|| args::message(method, "invalid generator ID"))
}

fn parse_id(value: String) -> Option<RandomGeneratorId> {
  let value = value.strip_prefix("rng_")?.parse::<u64>().ok()?;
  (value > 0).then_some(RandomGeneratorId(value))
}

fn format_id(id: RandomGeneratorId) -> String {
  format!("rng_{:03}", id.0)
}

fn auto_seed() -> u64 {
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |duration| duration.as_nanos() as u64);
  timestamp
    ^ (std::process::id() as u64).rotate_left(17)
    ^ AUTO_SEED_SEQUENCE
      .fetch_add(1, Ordering::Relaxed)
      .rotate_left(31)
}

fn with_direct_generator<R>(
  state: &SharedApiState,
  method: &str,
  operation: impl FnOnce(
    &RandomService,
    &mut crate::host_engine::services::RuntimeObjectPool,
    RandomGeneratorId,
  ) -> mlua::Result<R>,
) -> mlua::Result<R> {
  let existing = state.borrow().direct_random_id;
  with_pool_mut(state, method, |objects| {
    let service = RandomService::new();
    let id = existing
      .unwrap_or_else(|| service.create(objects.runtime_mut(), RandomSeed::U64(auto_seed())));
    if existing.is_none() {
      state.borrow_mut().direct_random_id = Some(id);
    }
    operation(&service, objects.runtime_mut(), id)
  })
}

fn with_pool<R>(
  state: &SharedApiState,
  method: &str,
  operation: impl FnOnce(&crate::host_engine::services::LuaObjectPool) -> mlua::Result<R>,
) -> mlua::Result<R> {
  let objects = state
    .borrow()
    .objects
    .upgrade()
    .ok_or_else(|| args::message(method, "session object pool is unavailable"))?;
  let objects = objects
    .try_borrow()
    .map_err(|_| args::message(method, "session object pool is busy"))?;
  operation(
    objects
      .as_ref()
      .ok_or_else(|| args::message(method, "session object pool is unavailable"))?,
  )
}

fn with_pool_mut<R>(
  state: &SharedApiState,
  method: &str,
  operation: impl FnOnce(&mut crate::host_engine::services::LuaObjectPool) -> mlua::Result<R>,
) -> mlua::Result<R> {
  let objects = state
    .borrow()
    .objects
    .upgrade()
    .ok_or_else(|| args::message(method, "session object pool is unavailable"))?;
  let mut objects = objects
    .try_borrow_mut()
    .map_err(|_| args::message(method, "session object pool is busy"))?;
  operation(
    objects
      .as_mut()
      .ok_or_else(|| args::message(method, "session object pool is unavailable"))?,
  )
}
