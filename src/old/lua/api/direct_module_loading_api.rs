// 模块加载 API，提供 load_function 函数，用于加载包内 scripts/function/ 目录下的 Lua 辅助脚本。该脚本必须返回一个 Lua 表，供游戏动态调用

use std::fs; // 读取脚本文件
use std::io::ErrorKind; // 区分文件不存在等错误
use std::path::{Component, PathBuf}; // 路径解析

use mlua::{Lua, Value, Variadic}; // Lua 类型

use crate::app::i18n; // 国际化错误
use crate::game::registry::PackageDescriptor; // 包描述符
use crate::lua::api::common; // 参数校验
use crate::lua::engine::RuntimeBridges; // 运行时桥接
use crate::utils::host_log; // 日志

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    // 读取并执行指定路径的 Lua 脚本，要求脚本执行后返回一个表；路径必须以 / 或 \ 开头，解析到 scripts/function/ 目录下
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

// 获取当前游戏包描述符
fn current_package(
    bridges: &RuntimeBridges,
) -> Option<&crate::game::registry::PackageDescriptor> {
    bridges.game.package_info()
}

// 将逻辑路径解析为 package.root_dir/scripts/function/ 下的绝对路径，禁止 .. 和空路径，要求文件存在
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

// 错误处理和日志记录
fn classify_load_error(path: &str, err: std::io::Error) -> mlua::Error {
    match err.kind() {
        ErrorKind::NotFound => file_not_found_error(path),
        _ => load_helper_script_failed_error(&err.to_string()),
    }
}

// 错误处理和日志记录
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

// 错误处理和日志记录
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

// 错误处理和日志记录
fn path_contains_parent_error() -> mlua::Error {
    host_log::append_host_error("host.exception.load_function_path_contains_parent", &[]);
    mlua::Error::external(i18n::t_or(
        "host.exception.load_function_path_contains_parent",
        "Path contains `..` operator, access denied",
    ))
}

// 错误处理和日志记录
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
