use std::fs;
use std::path::{Component, PathBuf};

use mlua::{Lua, Value};

use crate::game::registry::PackageDescriptor;
use crate::lua::engine::RuntimeBridges;

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    {
        let bridges = bridges.clone();
        globals.set(
            "load_function",
            lua.create_function(move |lua, path: String| {
                let Some(package) = current_package(&bridges) else {
                    return lua.create_table();
                };
                let Some(function_path) = resolve_function_path(package, &path) else {
                    return lua.create_table();
                };
                let Ok(source) = fs::read_to_string(&function_path) else {
                    return lua.create_table();
                };

                match lua
                    .load(source.trim_start_matches('\u{feff}'))
                    .set_name(function_path.to_string_lossy().as_ref())
                    .eval::<Value>()
                {
                    Ok(Value::Table(table)) => Ok(table),
                    Ok(_) => lua.create_table(),
                    Err(_) => lua.create_table(),
                }
            })?,
        )?;
    }

    Ok(())
}

fn current_package(
    bridges: &RuntimeBridges,
) -> Option<&crate::game::registry::PackageDescriptor> {
    bridges.game.package_info()
}

fn resolve_function_path(package: &PackageDescriptor, logical_path: &str) -> Option<PathBuf> {
    let trimmed = logical_path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let raw = PathBuf::from(trimmed);
    if raw.is_absolute() {
        return None;
    }

    let mut clean = PathBuf::new();
    for component in raw.components() {
        match component {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => return None,
        }
    }

    if clean.as_os_str().is_empty() {
        return None;
    }

    Some(package.root_dir.join("scripts").join("function").join(clean))
}
