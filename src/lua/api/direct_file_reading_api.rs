use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, PathBuf};

use csv::StringRecord;
use mlua::{Lua, Table, Value, Variadic};
use quick_xml::Reader;
use quick_xml::events::Event;
use serde_json::{Map, Value as JsonValue};

use crate::app::i18n;
use crate::game::registry::PackageDescriptor;
use crate::lua::api::common;
use crate::lua::api::direct_debug_api;
use crate::lua::engine::RuntimeBridges;
use crate::utils::host_log;

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    {
        let bridges = bridges.clone();
        globals.set(
            "translate",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let key = common::expect_string_arg(&args, 0, "key")?;
                translate_value(lua, &bridges, &key)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "read_bytes",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                let resolved = resolve_required_asset_path(&bridges, &path)?;
                let bytes = fs::read(&resolved).map_err(|err| classify_read_error(&path, err))?;
                let hex = bytes
                    .iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                Ok(Value::String(lua.create_string(&hex)?))
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "read_text",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                let resolved = resolve_required_asset_path(&bridges, &path)?;
                let text = fs::read_to_string(&resolved)
                    .map(|text| text.trim_start_matches('\u{feff}').to_string())
                    .map_err(|err| classify_text_read_error(&path, err))?;
                Ok(Value::String(lua.create_string(&text)?))
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "read_json",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                let text = read_text_file(&bridges, &path)?;
                let value = serde_json::from_str::<JsonValue>(&text)
                    .map_err(|_| invalid_file_format_error(&path))?;
                json_to_lua_value(lua, &value)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "read_json_string",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                let text = read_text_file(&bridges, &path)?;
                serde_json::from_str::<JsonValue>(&text)
                    .map_err(|_| invalid_file_format_error(&path))?;
                Ok(Value::String(lua.create_string(&text)?))
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "read_yaml",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                let text = read_text_file(&bridges, &path)?;
                let value = serde_yaml::from_str::<serde_yaml::Value>(&text)
                    .map_err(|_| invalid_file_format_error(&path))?;
                let value = serde_json::to_value(value)
                    .map_err(|_| invalid_file_format_error(&path))?;
                json_to_lua_value(lua, &value)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "read_yaml_string",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                let text = read_text_file(&bridges, &path)?;
                serde_yaml::from_str::<serde_yaml::Value>(&text)
                    .map_err(|_| invalid_file_format_error(&path))?;
                Ok(Value::String(lua.create_string(&text)?))
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "read_toml",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                let text = read_text_file(&bridges, &path)?;
                let value = text
                    .parse::<toml::Value>()
                    .map_err(|_| invalid_file_format_error(&path))?;
                let value = serde_json::to_value(value)
                    .map_err(|_| invalid_file_format_error(&path))?;
                json_to_lua_value(lua, &value)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "read_toml_string",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                let text = read_text_file(&bridges, &path)?;
                text.parse::<toml::Value>()
                    .map_err(|_| invalid_file_format_error(&path))?;
                Ok(Value::String(lua.create_string(&text)?))
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "read_csv",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                let text = read_text_file(&bridges, &path)?;
                let value = parse_csv_to_json(&text).map_err(|_| invalid_file_format_error(&path))?;
                json_to_lua_value(lua, &value)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "read_csv_string",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                let text = read_text_file(&bridges, &path)?;
                parse_csv_to_json(&text).map_err(|_| invalid_file_format_error(&path))?;
                Ok(Value::String(lua.create_string(&text)?))
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "read_xml",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                let text = read_text_file(&bridges, &path)?;
                let value = parse_xml_to_json(&text).map_err(|_| invalid_file_format_error(&path))?;
                json_to_lua_value(lua, &value)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "read_xml_string",
            lua.create_function(move |lua, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                let text = read_text_file(&bridges, &path)?;
                parse_xml_to_json(&text).map_err(|_| invalid_file_format_error(&path))?;
                Ok(Value::String(lua.create_string(&text)?))
            })?,
        )?;
    }

    Ok(())
}

fn translate_value(lua: &Lua, bridges: &RuntimeBridges, key: &str) -> mlua::Result<Value> {
    let package = current_package(bridges).ok_or_else(|| translate_failed_error(key))?;
    let normalized_key = key.trim();
    if normalized_key.is_empty() {
        return Err(translate_failed_error(key));
    }

    let current_code = i18n::current_language_code().replace('-', "_").to_lowercase();
    match load_package_lang_value_result(package, &current_code, normalized_key) {
        Ok(Some(value)) => return Ok(Value::String(lua.create_string(&value)?)),
        Ok(None) => {
            write_script_warning(
                bridges,
                &i18n::t_or(
                    "script.warning.translate_fallback_to_en_us",
                    "Language key for current host language not found, falling back to `en_us.json`",
                ),
            );
        }
        Err(_) => return Err(translate_failed_error(normalized_key)),
    }

    match load_package_lang_value_result(package, "en_us", normalized_key) {
        Ok(Some(value)) => Ok(Value::String(lua.create_string(&value)?)),
        Ok(None) => {
            let message = i18n::t_or(
                "script.warning.translate_missing_in_en_us",
                "Language key not found in `en_us.json`: [missing-i18n-key: `{key}`]",
            )
            .replace("{key}", normalized_key);
            write_script_warning(bridges, &message);
            let missing = format!("[missing-i18n-key: {}]", normalized_key);
            Ok(Value::String(lua.create_string(&missing)?))
        }
        Err(_) => Err(translate_failed_error(normalized_key)),
    }
}

fn current_package(bridges: &RuntimeBridges) -> Option<&PackageDescriptor> {
    bridges.game.package_info()
}

fn read_text_file(bridges: &RuntimeBridges, path: &str) -> mlua::Result<String> {
    let resolved = resolve_required_asset_path(bridges, path)?;
    fs::read_to_string(&resolved)
        .map(|text| text.trim_start_matches('\u{feff}').to_string())
        .map_err(|err| classify_text_read_error(path, err))
}

fn resolve_required_asset_path(bridges: &RuntimeBridges, logical_path: &str) -> mlua::Result<PathBuf> {
    let package = current_package(bridges).ok_or_else(|| file_not_found_error(logical_path))?;
    resolve_absolute_asset_path(package, logical_path)
}

fn resolve_absolute_asset_path(package: &PackageDescriptor, logical_path: &str) -> mlua::Result<PathBuf> {
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

    let resolved = package.root_dir.join("assets").join(clean);
    if !resolved.exists() {
        return Err(file_not_found_error(trimmed));
    }
    Ok(resolved)
}

fn load_package_lang_value_result(
    package: &PackageDescriptor,
    code: &str,
    key: &str,
) -> Result<Option<String>, ()> {
    let lang_path = package
        .root_dir
        .join("assets")
        .join("lang")
        .join(format!("{code}.json"));
    if !lang_path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(lang_path).map_err(|_| ())?;
    let json = serde_json::from_str::<JsonValue>(raw.trim_start_matches('\u{feff}')).map_err(|_| ())?;
    Ok(json
        .as_object()
        .and_then(|object| object.get(key))
        .and_then(JsonValue::as_str)
        .map(|value| value.to_string()))
}

fn parse_csv_to_json(text: &str) -> Result<JsonValue, csv::Error> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(text.as_bytes());
    let mut rows = Vec::new();
    for record in reader.records() {
        rows.push(csv_record_to_json(record?));
    }
    Ok(JsonValue::Array(rows))
}

fn csv_record_to_json(record: StringRecord) -> JsonValue {
    JsonValue::Array(
        record
            .iter()
            .map(|field| JsonValue::String(field.to_string()))
            .collect(),
    )
}

fn parse_xml_to_json(text: &str) -> Result<JsonValue, ()> {
    let mut reader = Reader::from_str(text);
    reader.config_mut().trim_text(true);

    let mut stack: Vec<XmlNode> = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                stack.push(XmlNode {
                    tag: String::from_utf8_lossy(event.name().as_ref()).to_string(),
                    attributes: attributes_to_map(&event),
                    children: Vec::new(),
                    text: String::new(),
                });
            }
            Ok(Event::Empty(event)) => {
                let node = XmlNode {
                    tag: String::from_utf8_lossy(event.name().as_ref()).to_string(),
                    attributes: attributes_to_map(&event),
                    children: Vec::new(),
                    text: String::new(),
                };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    return Ok(xml_node_to_json(node));
                }
            }
            Ok(Event::Text(event)) => {
                if let Some(node) = stack.last_mut()
                    && let Ok(decoded) = event.decode()
                {
                    node.text.push_str(&decoded);
                }
            }
            Ok(Event::CData(event)) => {
                if let Some(node) = stack.last_mut()
                    && let Ok(decoded) = event.decode()
                {
                    node.text.push_str(&decoded);
                }
            }
            Ok(Event::End(_)) => {
                if let Some(node) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(node);
                    } else {
                        return Ok(xml_node_to_json(node));
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => return Err(()),
            _ => {}
        }
    }
    Err(())
}

fn attributes_to_map(event: &quick_xml::events::BytesStart<'_>) -> BTreeMap<String, String> {
    event
        .attributes()
        .flatten()
        .map(|attr| {
            (
                String::from_utf8_lossy(attr.key.as_ref()).to_string(),
                String::from_utf8_lossy(attr.value.as_ref()).to_string(),
            )
        })
        .collect()
}

#[derive(Default)]
struct XmlNode {
    tag: String,
    attributes: BTreeMap<String, String>,
    children: Vec<XmlNode>,
    text: String,
}

fn xml_node_to_json(node: XmlNode) -> JsonValue {
    let mut object = Map::new();
    object.insert("tag".to_string(), JsonValue::String(node.tag));
    object.insert(
        "attributes".to_string(),
        JsonValue::Object(
            node.attributes
                .into_iter()
                .map(|(key, value)| (key, JsonValue::String(value)))
                .collect(),
        ),
    );
    object.insert(
        "children".to_string(),
        JsonValue::Array(node.children.into_iter().map(xml_node_to_json).collect()),
    );
    object.insert("text".to_string(), JsonValue::String(node.text));
    JsonValue::Object(object)
}

fn json_to_lua_value(lua: &Lua, value: &JsonValue) -> mlua::Result<Value> {
    match value {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(value) => Ok(Value::Boolean(*value)),
        JsonValue::Number(value) => {
            if let Some(value) = value.as_i64() {
                Ok(Value::Integer(value))
            } else if let Some(value) = value.as_f64() {
                Ok(Value::Number(value))
            } else {
                Ok(Value::Nil)
            }
        }
        JsonValue::String(value) => Ok(Value::String(lua.create_string(value)?)),
        JsonValue::Array(items) => {
            let table = lua.create_table()?;
            for (index, item) in items.iter().enumerate() {
                table.set(index + 1, json_to_lua_value(lua, item)?)?;
            }
            Ok(Value::Table(table))
        }
        JsonValue::Object(object) => {
            let table: Table = lua.create_table()?;
            for (key, item) in object {
                table.set(key.as_str(), json_to_lua_value(lua, item)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

fn file_not_found_error(path: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.target_file_not_found", &[("path", path)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.target_file_not_found",
            "Target file not found: `{path}`",
        )
        .replace("{path}", path),
    )
}

fn invalid_path_format_error(path: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.invalid_read_path_format", &[("path", path)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.invalid_read_path_format",
            "Invalid path format: expected absolute path, got `{path}`",
        )
        .replace("{path}", path),
    )
}

fn path_contains_parent_error() -> mlua::Error {
    host_log::append_host_error("host.exception.read_path_contains_parent", &[]);
    mlua::Error::external(i18n::t_or(
        "host.exception.read_path_contains_parent",
        "Path contains `..` operator, access denied",
    ))
}

fn invalid_file_format_error(path: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.invalid_file_format", &[("path", path)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.invalid_file_format",
            "Invalid file format, parsing failed: `{path}`",
        )
        .replace("{path}", path),
    )
}

fn translate_failed_error(key: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.translate_retrieve_failed", &[("key", key)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.translate_retrieve_failed",
            "Failed to retrieve content for language key: `{key}`",
        )
        .replace("{key}", key),
    )
}

fn classify_read_error(path: &str, err: std::io::Error) -> mlua::Error {
    match err.kind() {
        ErrorKind::NotFound => file_not_found_error(path),
        _ => read_file_failed_error(&err.to_string()),
    }
}

fn classify_text_read_error(path: &str, err: std::io::Error) -> mlua::Error {
    match err.kind() {
        ErrorKind::NotFound => file_not_found_error(path),
        ErrorKind::InvalidData => invalid_file_format_error(path),
        _ => read_file_failed_error(&err.to_string()),
    }
}

fn read_file_failed_error(err: &str) -> mlua::Error {
    host_log::append_host_error("host.exception.read_file_failed", &[("err", err)]);
    mlua::Error::external(
        i18n::t_or(
            "host.exception.read_file_failed",
            "Failed to read file: {err}",
        )
        .replace("{err}", err),
    )
}

fn write_script_warning(bridges: &RuntimeBridges, message: &str) {
    let _ = direct_debug_api::write_log_line(
        bridges,
        &i18n::t_or("debug.title.warning", "Warning"),
        message,
    );
}
