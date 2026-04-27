/// 模块加载 API，加载包内 scripts/function/ 目录下的 Lua 辅助脚本
/// 业务逻辑：
/// 加载指定路径的 Lua 文件，执行后必须返回 table

use std::fs;
use std::io::ErrorKind;
use std::path::{Component, PathBuf};

use mlua::{Lua, Value, Variadic};

use crate::app::i18n;
use crate::game::registry::PackageDescriptor;
use crate::lua::api::common;
use crate::lua::engine::RuntimeBridges;
use crate::utils::host_log;

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    {
        let bridges = bridges.clone();
        globals.set(
            "load_function",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                let Some(package) = current_package(&bridges) else {
                    return Err(load_helper_script_failed_error("no active package"));
                };
                let function_path = resolve_function_path(package, &path)?;
                let source = fs::read_to_string(&function_path)
                    .map_err(|err| classify_load_error(&path, err))?;

                match lua
                    .load(source.trim_start_matches('\u{feff}'))
                    .set_name(function_path.to_string_lossy().as_ref())
                    .eval::<Value>()
                {
                    Ok(Value::Table(table)) => Ok(table),
                    Ok(_) => Err(load_helper_script_failed_error(
                        "helper script must return a table",
                    )),
                    Err(err) => Err(load_helper_script_failed_error(&err.to_string())),
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

fn resolve_function_path(package: &PackageDescriptor, logical_path: &str) -> mlua::Result<PathBuf> {
    let trimmed = logical_path.trim();
    if !trimmed.starts_with('/') && !trimmed.starts_with('\\') {
        return Err(invalid_path_format_error(trimmed));
    }

    let stripped = trimmed.trim_start_matches(['/', '\\']);
    let mut clean = PathBuf::new();
    for component in PathBuf::from(stripped).components() {
        match component {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::ParentDir => return Err(path_contains_parent_error()),
            Component::Prefix(_) | Component::RootDir => return Err(invalid_path_format_error(trimmed)),
        }
    }

    if clean.as_os_str().is_empty() {
        return Err(invalid_path_format_error(trimmed));
    }

    let resolved = package.root_dir.join("scripts").join("function").join(clean);
    if !resolved.exists() {
        return Err(file_not_found_error(trimmed));
    }
    Ok(resolved)
}

fn classify_load_error(path: &str, err: std::io::Error) -> mlua::Error {
    match err.kind() {
        ErrorKind::NotFound => file_not_found_error(path),
        _ => load_helper_script_failed_error(&err.to_string()),
    }
}

fn file_not_found_error(path: &str) -> mlua::Error {
    host_log::append_host_error(
        "host.exception.load_function_target_file_not_found",
        &[("path", path)],
    );
    mlua::Error::external(
        i18n::t_or(
            "host.exception.load_function_target_file_not_found",
            "Target file not found: {path}",
        )
        .replace("{path}", path),
    )
}

fn invalid_path_format_error(path: &str) -> mlua::Error {
    host_log::append_host_error(
        "host.exception.load_function_invalid_path_format",
        &[("path", path)],
    );
    mlua::Error::external(
        i18n::t_or(
            "host.exception.load_function_invalid_path_format",
            "Invalid path format: expected absolute path, got `{path}`",
        )
        .replace("{path}", path),
    )
}

fn path_contains_parent_error() -> mlua::Error {
    host_log::append_host_error("host.exception.load_function_path_contains_parent", &[]);
    mlua::Error::external(i18n::t_or(
        "host.exception.load_function_path_contains_parent",
        "Path contains `..` operator, access denied",
    ))
}

fn load_helper_script_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error(
        "host.exception.load_helper_script_failed",
        &[("err", err)],
    );
    mlua::Error::external(
        i18n::t_or(
            "host.exception.load_helper_script_failed",
            "Failed to load helper script: {err}",
        )
        .replace("{err}", err),
    )
}
