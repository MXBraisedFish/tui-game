use std::fs;
use std::path::{Component, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use mlua::{Lua, Value, Variadic};

use crate::app::i18n;
use crate::game::registry::{GameSourceKind, PackageDescriptor};
use crate::lua::api::common;
use crate::lua::engine::RuntimeBridges;
use crate::mods;
use crate::utils::{host_log, path_utils};

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    install_writer(lua, &globals, "write_bytes", bridges.clone(), |path, content| {
        write_bytes(path, content)
    })?;
    install_writer(lua, &globals, "write_text", bridges.clone(), |path, content| {
        write_text(path, content)
    })?;
    install_writer(lua, &globals, "write_json", bridges.clone(), |path, content| {
        write_text(path, content)
    })?;
    install_writer(lua, &globals, "write_xml", bridges.clone(), |path, content| {
        write_text(path, content)
    })?;
    install_writer(lua, &globals, "write_yaml", bridges.clone(), |path, content| {
        write_text(path, content)
    })?;
    install_writer(lua, &globals, "write_toml", bridges.clone(), |path, content| {
        write_text(path, content)
    })?;
    install_writer(lua, &globals, "write_csv", bridges.clone(), |path, content| {
        write_text(path, content)
    })?;

    Ok(())
}

fn install_writer<F>(
    lua: &Lua,
    globals: &mlua::Table,
    api_name: &'static str,
    bridges: RuntimeBridges,
    write_fn: F,
) -> mlua::Result<()>
where
    F: Fn(&std::path::Path, &str) -> std::io::Result<()> + Clone + Send + 'static,
{
    let writer = write_fn.clone();
    globals.set(
        api_name,
        lua.create_function(move |_, args: Variadic<Value>| {
            common::expect_exact_arg_count(&args, 2)?;
            let path = common::expect_string_arg(&args, 0, "path")?;
            let content = common::expect_string_arg(&args, 1, "content")?;
            let allowed = is_write_allowed(&bridges);
            log_write_request(&bridges, api_name, &path, allowed);
            if !allowed {
                return Ok(false);
            }

            let Some(package) = current_package(&bridges) else {
                return Ok(false);
            };
            let Some(resolved) = resolve_asset_write_path(package, &path) else {
                return Ok(false);
            };
            if path_utils::ensure_parent_dir(&resolved).is_err() {
                return Ok(false);
            }

            Ok(writer(&resolved, &content).is_ok())
        })?,
    )?;
    Ok(())
}

fn current_package(
    bridges: &RuntimeBridges,
) -> Option<&crate::game::registry::PackageDescriptor> {
    bridges.game.package_info()
}

fn is_write_allowed(bridges: &RuntimeBridges) -> bool {
    if !bridges.game.write {
        return false;
    }

    let Some(package) = bridges.game.package_info() else {
        return false;
    };

    match package.source {
        GameSourceKind::Official => true,
        GameSourceKind::Mod => !is_mod_safe_mode_enabled(&package.namespace),
    }
}

fn is_mod_safe_mode_enabled(namespace: &str) -> bool {
    mods::load_mod_state()
        .mods
        .get(namespace)
        .map(|entry| {
            entry
                .session_safe_mode_enabled
                .unwrap_or(entry.safe_mode_enabled)
        })
        .unwrap_or(true)
}

fn resolve_asset_write_path(package: &PackageDescriptor, logical_path: &str) -> Option<PathBuf> {
    let trimmed = logical_path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        return None;
    }

    let mut clean = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => return None,
        }
    }

    if clean.as_os_str().is_empty() {
        return None;
    }

    Some(package.root_dir.join("assets").join(clean))
}

fn write_bytes(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    fs::write(path, content.as_bytes())
}

fn write_text(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    fs::write(path, content)
}

fn log_write_request(bridges: &RuntimeBridges, api_name: &str, path: &str, allowed: bool) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();
    let status_key = if allowed {
        "host.status.allowed"
    } else {
        "host.status.denied"
    };
    let status_fallback = if allowed { "allowed" } else { "denied" };
    let status = i18n::t_or(status_key, status_fallback);
    host_log::append_host_warning(
        "host.warning.file_write_request",
        &[
            ("game_uid", bridges.game.id.as_str()),
            ("timestamp", &timestamp),
            ("api", api_name),
            ("path", path),
            ("status", &status),
        ],
    );
}
