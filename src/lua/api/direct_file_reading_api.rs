use std::collections::BTreeMap;

use csv::StringRecord;
use mlua::{Lua, Table, Value, Variadic};
use quick_xml::Reader;
use quick_xml::events::Event;
use serde_json::{Map, Value as JsonValue};

use crate::game::resources;
use crate::lua::api::common;
use crate::lua::engine::RuntimeBridges;

pub(crate) fn install(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    {
        let bridges = bridges.clone();
        globals.set(
            "translate",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let key = common::expect_string_arg(&args, 0, "key")?;
                Ok(current_package(&bridges)
                    .map(|package| resources::resolve_package_text(package, &key))
                    .unwrap_or(key))
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
                let bytes = current_package(&bridges)
                    .and_then(|package| resources::read_package_bytes(package, &path).ok())
                    .unwrap_or_default();
                let hex = bytes
                    .iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                lua.create_string(&hex)
            })?,
        )?;
    }

    {
        let bridges = bridges.clone();
        globals.set(
            "read_text",
            lua.create_function(move |_, args: Variadic<Value>| {
                common::expect_exact_arg_count(&args, 1)?;
                let path = common::expect_string_arg(&args, 0, "path")?;
                Ok(current_package(&bridges)
                    .and_then(|package| resources::read_package_text(package, &path).ok())
                    .unwrap_or_default())
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
                let value = current_package(&bridges)
                    .and_then(|package| resources::read_package_json(package, &path).ok())
                    .unwrap_or(JsonValue::Null);
                json_to_lua_value(lua, &value)
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
                let value = current_package(&bridges)
                    .and_then(|package| resources::read_package_text(package, &path).ok())
                    .and_then(|text| serde_yaml::from_str::<serde_yaml::Value>(&text).ok())
                    .and_then(|value| serde_json::to_value(value).ok())
                    .unwrap_or(JsonValue::Null);
                json_to_lua_value(lua, &value)
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
                let value = current_package(&bridges)
                    .and_then(|package| resources::read_package_text(package, &path).ok())
                    .and_then(|text| text.parse::<toml::Value>().ok())
                    .and_then(|value| serde_json::to_value(value).ok())
                    .unwrap_or(JsonValue::Null);
                json_to_lua_value(lua, &value)
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
                let value = current_package(&bridges)
                    .and_then(|package| resources::read_package_text(package, &path).ok())
                    .map(|text| parse_csv_to_json(&text))
                    .unwrap_or(JsonValue::Null);
                json_to_lua_value(lua, &value)
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
                let value = current_package(&bridges)
                    .and_then(|package| resources::read_package_text(package, &path).ok())
                    .map(|text| parse_xml_to_json(&text))
                    .unwrap_or(JsonValue::Null);
                json_to_lua_value(lua, &value)
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

fn parse_csv_to_json(text: &str) -> JsonValue {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(text.as_bytes());
    let rows = reader
        .records()
        .filter_map(Result::ok)
        .map(csv_record_to_json)
        .collect::<Vec<_>>();
    JsonValue::Array(rows)
}

fn csv_record_to_json(record: StringRecord) -> JsonValue {
    JsonValue::Array(
        record
            .iter()
            .map(|field| JsonValue::String(field.to_string()))
            .collect(),
    )
}

fn parse_xml_to_json(text: &str) -> JsonValue {
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
                    return xml_node_to_json(node);
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
                        return xml_node_to_json(node);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    JsonValue::Null
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
