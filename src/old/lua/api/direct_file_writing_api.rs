// 文件写入 API，受游戏清单中的 write 标志和 Mod 安全模式控制。提供 write_bytes, write_text, write_json, write_xml, write_yaml, write_toml, write_csv 等函数，用于将内容写入包内 assets/ 目录下的文件。所有写入操作需经过安全校验，并记录审计日志

use std::fs; // 文件写入操作
use std::io::ErrorKind; // 区分错误类型
use std::path::{Component, PathBuf}; // 路径解析和规范化
use std::time::{SystemTime, UNIX_EPOCH}; // 生成时间戳用于日志

use mlua::{Lua, Value, Variadic}; // Lua 类型和参数

use crate::app::i18n; // 国际化错误消息
use crate::game::registry::{GameSourceKind, PackageDescriptor}; // 区分官方/Mod 包
use crate::lua::api::common; // 参数校验
use crate::lua::engine::RuntimeBridges; // 运行时桥接
use crate::mods; // 读取 Mod 安全模式状态
use crate::utils::{host_log, path_utils}; // 日志和路径工具

// 注册 write_bytes, write_text, write_json, write_xml, write_yaml, write_toml, write_csv 等函数到全局表
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

// 泛型辅助函数，创建具体的写入 API 闭包，进行参数校验、权限检查、路径解析和错误处理
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
            let logged_path = resolve_logged_path(&bridges, &path);
            let allowed = is_write_allowed(&bridges);
            log_write_request(&bridges, api_name, &logged_path, allowed);
            if !allowed {
                return Ok(false);
            }

            let Some(package) = current_package(&bridges) else {
                return Ok(false);
            };
            let resolved = resolve_asset_write_path(package, &path)?;
            path_utils::ensure_parent_dir(&resolved)
                .map_err(|err| write_file_failed_error(&err.to_string()))?;

            writer(&resolved, &content)
                .map_err(|err| classify_write_error(err, &path))?;
            Ok(true)
        })?,
    )?;
    Ok(())
}

fn current_package(
    bridges: &RuntimeBridges,
) -> Option<&crate::game::registry::PackageDescriptor> {
    bridges.game.package_info()
}

// 检查写入是否允许：需要 game.write == true；官方游戏直接允许；Mod 游戏需检查是否处于安全模式（safe_mode_enabled 或 session_safe_mode_enabled）
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

// 从 Mod 状态中读取安全模式标志，默认 true（即默认禁止写入）
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

// 将逻辑路径（必须以 / 或 \ 开头）解析为 package.root_dir/assets/ 下的绝对路径，禁止 .. 和空路径，确保不逃离包根目录
fn resolve_asset_write_path(package: &PackageDescriptor, logical_path: &str) -> mlua::Result<PathBuf> {
    let trimmed = logical_path.trim();
    if !trimmed.starts_with('/') && !trimmed.starts_with('\\') {
        return Err(invalid_path_format_error(trimmed));
    }

    let stripped = trimmed.trim_start_matches(['/', '\\']);
    let path = PathBuf::from(stripped);
    let mut clean = PathBuf::new();
    for component in path.components() {
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

    Ok(package.root_dir.join("assets").join(clean))
}

// 将字符串 content 以字节形式写入文件（字节序不变）
fn write_bytes(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    fs::write(path, content.as_bytes())
}

// 将字符串内容按 UTF-8 写入文本文件
fn write_text(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    fs::write(path, content)
}

// 用于日志记录：尝试解析路径，成功则显示绝对路径，否则显示原始逻辑路径
fn resolve_logged_path(bridges: &RuntimeBridges, logical_path: &str) -> String {
    current_package(bridges)
        .and_then(|package| resolve_asset_write_path(package, logical_path).ok())
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| logical_path.to_string())
}

// 记录文件写入请求的审计日志，包含时间戳、游戏 ID、API 名称、路径和状态（允许/拒绝）
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

// 生成各类 Lua 错误，记录日志
fn invalid_path_format_error(path: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.invalid_write_path_format", &[("path", path)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.invalid_write_path_format",
            "Invalid path format: expected absolute path, got `{path}`",
        )
        .replace("{path}", path),
    )
}

// 生成各类 Lua 错误，记录日志
fn path_contains_parent_error() -> mlua::Error {
    host_log::append_host_error("host.exception.write_path_contains_parent", &[]);
    mlua::Error::external(i18n::t_or(
        "host.exception.write_path_contains_parent",
        "Path contains `..` operator, access denied",
    ))
}

// 生成各类 Lua 错误，记录日志
fn classify_write_error(err: std::io::Error, path: &str) -> mlua::Error {
    match err.kind() {
        ErrorKind::NotFound => write_file_failed_error(&format!("{} ({path})", err)),
        _ => write_file_failed_error(&err.to_string()),
    }
}

// 生成各类 Lua 错误，记录日志
fn write_file_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.write_file_failed", &[("err", err)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.write_file_failed",
            "Failed to write file: {err}",
        )
        .replace("{err}", err),
    )
}
