//! 直用式随机数 API 公开

use mlua::{Lua, Value, Variadic};

use super::random_support::random_parser::{self, RandomIntArgs};
use super::random_support::random_store::{DEFAULT_RANDOM_MAX, RandomKind, RandomStore};
use super::random_support::random_table;
use super::scope::ApiScope;
use super::validation::argument;
use crate::host_engine::boot::preload::lua_runtime::HostLuaBridge;

/// 安装随机数 API。
pub fn install(lua: &Lua, api_scope: ApiScope, host_bridge: HostLuaBridge) -> mlua::Result<()> {
    if !api_scope.allows_random() {
        return Ok(());
    }

    let globals = lua.globals();
    install_random(lua, &globals, host_bridge.clone())?;
    install_random_float(lua, &globals, host_bridge.clone())?;
    install_random_create(lua, &globals, host_bridge.clone())?;
    install_random_float_create(lua, &globals, host_bridge.clone())?;
    install_random_reset_step(lua, &globals, host_bridge.clone())?;
    install_random_kill(lua, &globals, host_bridge.clone())?;
    install_set_random_note(lua, &globals, host_bridge.clone())?;
    install_get_random_list(lua, &globals, host_bridge.clone())?;
    install_get_random_info(lua, &globals, host_bridge.clone())?;
    install_get_random_step(lua, &globals, host_bridge.clone())?;
    install_get_random_seed(lua, &globals, host_bridge.clone())?;
    install_get_random_type(lua, &globals, host_bridge)?;

    Ok(())
}

fn install_random(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "random",
        lua.create_function(move |_, args: Variadic<Value>| {
            let random_args = random_parser::parse_random_int_args(&args)?;
            match random_args {
                RandomIntArgs::Default => {
                    Ok(RandomStore::default_random_int(0, DEFAULT_RANDOM_MAX))
                }
                RandomIntArgs::Max(max) => random_with_default_store(0, max),
                RandomIntArgs::Range { min, max } => random_with_default_store(min, max),
                RandomIntArgs::Id(id) => host_bridge.with_random_store(|random_store| {
                    let random_entry = random_store.random_mut(id.as_str())?;
                    ensure_random_kind(random_entry.kind, RandomKind::Int)?;
                    Ok(random_entry.next_int(0, DEFAULT_RANDOM_MAX))
                }),
                RandomIntArgs::MaxWithId { max, id } => {
                    validate_random_range(0, max)?;
                    host_bridge.with_random_store(|random_store| {
                        let random_entry = random_store.random_mut(id.as_str())?;
                        ensure_random_kind(random_entry.kind, RandomKind::Int)?;
                        Ok(random_entry.next_int(0, max))
                    })
                }
                RandomIntArgs::RangeWithId { min, max, id } => {
                    validate_random_range(min, max)?;
                    host_bridge.with_random_store(|random_store| {
                        let random_entry = random_store.random_mut(id.as_str())?;
                        ensure_random_kind(random_entry.kind, RandomKind::Int)?;
                        Ok(random_entry.next_int(min, max))
                    })
                }
            }
        })?,
    )
}

fn install_random_float(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "random_float",
        lua.create_function(move |_, args: Variadic<Value>| {
            let random_id = random_parser::parse_random_float_id(&args)?;
            match random_id {
                Some(id) => host_bridge.with_random_store(|random_store| {
                    let random_entry = random_store.random_mut(id.as_str())?;
                    ensure_random_kind(random_entry.kind, RandomKind::Float)?;
                    Ok(random_entry.next_float())
                }),
                None => Ok(RandomStore::default_random_float()),
            }
        })?,
    )
}

fn install_random_create(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_random_creator(lua, globals, host_bridge, "random_create", RandomKind::Int)
}

fn install_random_float_create(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_random_creator(
        lua,
        globals,
        host_bridge,
        "random_float_create",
        RandomKind::Float,
    )
}

fn install_random_creator(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
    function_name: &'static str,
    random_kind: RandomKind,
) -> mlua::Result<()> {
    globals.set(
        function_name,
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_arg_count_range(&args, 1, 2)?;
            let seed = argument::expect_string_arg(&args, 0)?;
            let note = argument::expect_optional_string_arg(&args, 1)?.unwrap_or_default();
            let id = host_bridge.with_random_store(|random_store| {
                random_store.create_random(seed, note, random_kind)
            })?;
            Ok(Value::String(lua.create_string(id.as_str())?))
        })?,
    )
}

fn install_random_reset_step(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "random_reset_step",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let id = argument::expect_string_arg(&args, 0)?;
            host_bridge.with_random_store(|random_store| {
                random_store.random_mut(id.as_str())?.reset_step();
                Ok(())
            })
        })?,
    )
}

fn install_random_kill(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "random_kill",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let id = argument::expect_string_arg(&args, 0)?;
            host_bridge.with_random_store(|random_store| random_store.kill_random(id.as_str()))
        })?,
    )
}

fn install_set_random_note(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "set_random_note",
        lua.create_function(move |_, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 2)?;
            let id = argument::expect_string_arg(&args, 0)?;
            let note = argument::expect_string_arg(&args, 1)?;
            host_bridge.with_random_store(|random_store| {
                random_store.random_mut(id.as_str())?.set_note(note);
                Ok(())
            })
        })?,
    )
}

fn install_get_random_list(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_random_list",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 0)?;
            host_bridge.with_random_store(|random_store| {
                let table = lua.create_table()?;
                for (index, random_entry) in random_store.randoms().enumerate() {
                    table.set(
                        index + 1,
                        random_table::build_random_info_table(lua, random_entry)?,
                    )?;
                }
                Ok(table)
            })
        })?,
    )
}

fn install_get_random_info(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    globals.set(
        "get_random_info",
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let id = argument::expect_string_arg(&args, 0)?;
            host_bridge.with_random_store(|random_store| {
                random_table::build_random_info_table(lua, random_store.random(id.as_str())?)
            })
        })?,
    )
}

fn install_get_random_step(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_random_value_getter(
        lua,
        globals,
        host_bridge,
        "get_random_step",
        |_lua, random_entry| Ok(Value::Integer(random_entry.step as i64)),
    )
}

fn install_get_random_seed(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_random_value_getter(
        lua,
        globals,
        host_bridge,
        "get_random_seed",
        |lua, random_entry| {
            Ok(Value::String(
                lua.create_string(random_entry.seed.as_str())?,
            ))
        },
    )
}

fn install_get_random_type(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
) -> mlua::Result<()> {
    install_random_value_getter(
        lua,
        globals,
        host_bridge,
        "get_random_type",
        |lua, random_entry| {
            Ok(Value::String(
                lua.create_string(random_entry.kind.as_str())?,
            ))
        },
    )
}

fn install_random_value_getter(
    lua: &Lua,
    globals: &mlua::Table,
    host_bridge: HostLuaBridge,
    function_name: &'static str,
    getter: fn(&Lua, &super::random_support::random_store::RandomEntry) -> mlua::Result<Value>,
) -> mlua::Result<()> {
    globals.set(
        function_name,
        lua.create_function(move |lua, args: Variadic<Value>| {
            argument::expect_exact_arg_count(&args, 1)?;
            let id = argument::expect_string_arg(&args, 0)?;
            host_bridge
                .with_random_store(|random_store| getter(lua, random_store.random(id.as_str())?))
        })?,
    )
}

fn random_with_default_store(min: i64, max: i64) -> mlua::Result<i64> {
    validate_random_range(min, max)?;
    Ok(RandomStore::default_random_int(min, max))
}

fn validate_random_range(min: i64, max: i64) -> mlua::Result<()> {
    if max < min {
        Err(mlua::Error::external(format!(
            "random max must be greater than or equal to min: min={min}, max={max}"
        )))
    } else {
        Ok(())
    }
}

fn ensure_random_kind(actual_kind: RandomKind, expected_kind: RandomKind) -> mlua::Result<()> {
    if actual_kind == expected_kind {
        Ok(())
    } else {
        Err(mlua::Error::external("random generator type mismatch"))
    }
}
