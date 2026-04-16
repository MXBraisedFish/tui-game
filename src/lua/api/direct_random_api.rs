use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use mlua::{Lua, Table, Value, Variadic};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::lua::api::common;
use crate::lua::engine::RuntimeBridges;

const MAX_RANDOMS: usize = 64;
const DEFAULT_RANDOM_MAX: i64 = 2_147_483_647;

#[derive(Default)]
pub struct RandomStore {
    next_id: u64,
    randoms: BTreeMap<String, RandomEntry>,
}

struct RandomEntry {
    id: String,
    note: String,
    seed: String,
    step: u64,
    kind: RandomKind,
    rng: StdRng,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum RandomKind {
    Int,
    Float,
}

impl RandomKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Float => "float",
        }
    }
}

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    {
        let bridges = bridges.clone();
        globals.set(
            "random",
            lua.create_function(move |_, args: Variadic<Value>| random_call(&bridges, &args))?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "random_float",
            lua.create_function(move |_, args: Variadic<Value>| random_float_call(&bridges, &args))?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "random_create",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_arg_count_range(&args, 1, 2)?;
                let seed = common::expect_string_arg(&args, 0, "seed")?;
                let note = common::expect_optional_string_arg(&args, 1, "note")?;
                create_random(lua, &bridges, seed, note, RandomKind::Int)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "random_float_create",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_arg_count_range(&args, 1, 2)?;
                let seed = common::expect_string_arg(&args, 0, "seed")?;
                let note = common::expect_optional_string_arg(&args, 1, "note")?;
                create_random(lua, &bridges, seed, note, RandomKind::Float)
            })?,
        )?;
    }

    install_random_mutator(lua, &globals, "random_reset_step", bridges.clone(), |entry| {
        entry.step = 0;
        entry.rng = seeded_rng(&entry.seed);
    })?;

    {
        let bridges = bridges.clone();
        globals.set(
            "random_kill",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let id = common::expect_string_arg(&args, 0, "id")?;
                let mut store = random_store(&bridges)?;
                store.randoms.remove(&id);
                Ok(())
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "set_random_note",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 2)?;
                let id = common::expect_string_arg(&args, 0, "id")?;
                let note = common::expect_string_arg(&args, 1, "note")?;
                let mut store = random_store(&bridges)?;
                let entry = get_random_mut(&mut store, &id)?;
                entry.note = note;
                Ok(())
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "get_random_list",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 0)?;
                let store = random_store(&bridges)?;
                let arr = lua.create_table()?;
                for (idx, entry) in store.randoms.values().enumerate() {
                    arr.set(idx + 1, build_random_info(lua, entry)?)?;
                }
                Ok(arr)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "get_random_info",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let id = common::expect_string_arg(&args, 0, "id")?;
                let store = random_store(&bridges)?;
                let entry = get_random(&store, &id)?;
                build_random_info(lua, entry)
            })?,
        )?;
    }

    install_random_getter(lua, &globals, "get_random_step", bridges.clone(), |entry, _lua| {
        Ok(Value::Integer(entry.step as i64))
    })?;
    install_random_getter(lua, &globals, "get_random_seed", bridges.clone(), |entry, lua| {
        Ok(Value::String(lua.create_string(entry.seed.as_str())?))
    })?;
    install_random_getter(lua, &globals, "get_random_type", bridges.clone(), |entry, lua| {
        Ok(Value::String(lua.create_string(entry.kind.as_str())?))
    })?;

    Ok(())
}

fn random_call(bridges: &RuntimeBridges, args: &[Value]) -> mlua::Result<i64> {
    let mut store = random_store(bridges)?;
    match args.len() {
        0 => {
            let mut rng = rand::thread_rng();
            Ok(rng.gen_range(0..=DEFAULT_RANDOM_MAX))
        }
        1 => {
            if let Some(id) = value_as_string(&args[0]) {
                let entry = get_random_mut(&mut store, &id)?;
                ensure_int(entry)?;
                let value = entry.rng.gen_range(0..=DEFAULT_RANDOM_MAX);
                entry.step += 1;
                Ok(value)
            } else {
                let max = value_as_i64(&args[0])
                    .ok_or_else(|| common::arg_type_error("max", "number|string", &args[0]))?;
                let mut rng = rand::thread_rng();
                Ok(rng.gen_range(0..=max))
            }
        }
        2 => {
            if let Some(id) = value_as_string(&args[1]) {
                let max = value_as_i64(&args[0])
                    .ok_or_else(|| common::arg_type_error("max", "number", &args[0]))?;
                let entry = get_random_mut(&mut store, &id)?;
                ensure_int(entry)?;
                let value = entry.rng.gen_range(0..=max);
                entry.step += 1;
                Ok(value)
            } else {
                let min = value_as_i64(&args[0])
                    .ok_or_else(|| common::arg_type_error("min", "number", &args[0]))?;
                let max = value_as_i64(&args[1])
                    .ok_or_else(|| common::arg_type_error("max", "number|string", &args[1]))?;
                let mut rng = rand::thread_rng();
                Ok(rng.gen_range(min..=max))
            }
        }
        3 => {
            let min = value_as_i64(&args[0])
                .ok_or_else(|| common::arg_type_error("min", "number", &args[0]))?;
            let max = value_as_i64(&args[1])
                .ok_or_else(|| common::arg_type_error("max", "number", &args[1]))?;
            let id = value_as_string(&args[2])
                .ok_or_else(|| common::arg_type_error("id", "string", &args[2]))?;
            let entry = get_random_mut(&mut store, &id)?;
            ensure_int(entry)?;
            let value = entry.rng.gen_range(min..=max);
            entry.step += 1;
            Ok(value)
        }
        _ => Err(common::arg_count_error("0-3", args.len())),
    }
}

fn random_float_call(bridges: &RuntimeBridges, args: &[Value]) -> mlua::Result<f64> {
    let mut store = random_store(bridges)?;
    match args.len() {
        0 => {
            let mut rng = rand::thread_rng();
            Ok(rng.gen_range(0.0..1.0))
        }
        1 => {
            let id = value_as_string(&args[0])
                .ok_or_else(|| common::arg_type_error("id", "string", &args[0]))?;
            let entry = get_random_mut(&mut store, &id)?;
            ensure_float(entry)?;
            let value = entry.rng.gen_range(0.0..1.0);
            entry.step += 1;
            Ok(value)
        }
        _ => Err(common::arg_count_error("0-1", args.len())),
    }
}

fn create_random(
    lua: &Lua,
    bridges: &RuntimeBridges,
    seed: String,
    note: Option<String>,
    kind: RandomKind,
) -> mlua::Result<Value> {
    let mut store = random_store(bridges)?;
    if store.randoms.len() >= MAX_RANDOMS {
        return Ok(Value::Nil);
    }
    store.next_id += 1;
    let id = format!("random_{}", store.next_id);
    store.randoms.insert(
        id.clone(),
        RandomEntry {
            id: id.clone(),
            note: note.unwrap_or_default(),
            seed: seed.clone(),
            step: 0,
            kind,
            rng: seeded_rng(&seed),
        },
    );
    Ok(Value::String(lua.create_string(&id)?))
}

fn install_random_mutator<F>(
    lua: &Lua,
    globals: &Table,
    name: &'static str,
    bridges: RuntimeBridges,
    mutator: F,
) -> mlua::Result<()>
where
    F: Fn(&mut RandomEntry) + Clone + Send + 'static,
{
    let apply = mutator.clone();
    globals.set(
        name,
        lua.create_function(move |_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 1)?;
            let id = common::expect_string_arg(&args, 0, "id")?;
            let mut store = random_store(&bridges)?;
            let entry = get_random_mut(&mut store, &id)?;
            apply(entry);
            Ok(())
        })?,
    )?;
    Ok(())
}

fn install_random_getter<F>(
    lua: &Lua,
    globals: &Table,
    name: &'static str,
    bridges: RuntimeBridges,
    getter: F,
) -> mlua::Result<()>
where
    F: Fn(&RandomEntry, &Lua) -> mlua::Result<Value> + Clone + Send + 'static,
{
    let get = getter.clone();
    globals.set(
        name,
        lua.create_function(move |lua, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 1)?;
            let id = common::expect_string_arg(&args, 0, "id")?;
            let store = random_store(&bridges)?;
            let entry = get_random(&store, &id)?;
            get(entry, lua)
        })?,
    )?;
    Ok(())
}

fn random_store<'a>(
    bridges: &'a RuntimeBridges,
) -> mlua::Result<std::sync::MutexGuard<'a, RandomStore>> {
    bridges
        .randoms
        .lock()
        .map_err(|_| mlua::Error::external("random store poisoned"))
}

fn get_random_mut<'a>(
    store: &'a mut RandomStore,
    id: &str,
) -> mlua::Result<&'a mut RandomEntry> {
    store
        .randoms
        .get_mut(id)
        .ok_or_else(|| mlua::Error::external("random generator not found"))
}

fn get_random<'a>(store: &'a RandomStore, id: &str) -> mlua::Result<&'a RandomEntry> {
    store
        .randoms
        .get(id)
        .ok_or_else(|| mlua::Error::external("random generator not found"))
}

fn ensure_int(entry: &RandomEntry) -> mlua::Result<()> {
    if entry.kind == RandomKind::Int {
        Ok(())
    } else {
        Err(mlua::Error::external("random generator type mismatch"))
    }
}

fn ensure_float(entry: &RandomEntry) -> mlua::Result<()> {
    if entry.kind == RandomKind::Float {
        Ok(())
    } else {
        Err(mlua::Error::external("random generator type mismatch"))
    }
}

fn seeded_rng(seed: &str) -> StdRng {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    StdRng::seed_from_u64(hasher.finish())
}

fn build_random_info(lua: &Lua, entry: &RandomEntry) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("id", entry.id.as_str())?;
    table.set("note", entry.note.as_str())?;
    table.set("seed", entry.seed.as_str())?;
    table.set("step", entry.step as i64)?;
    table.set("type", entry.kind.as_str())?;
    Ok(table)
}

fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => s.to_str().ok().map(|v| v.to_string()),
        _ => None,
    }
}

fn value_as_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Integer(v) => Some(*v),
        Value::Number(v) => Some(*v as i64),
        _ => None,
    }
}
