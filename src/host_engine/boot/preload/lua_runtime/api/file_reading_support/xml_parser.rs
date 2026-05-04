//! XML 解析

use std::collections::BTreeMap;

use quick_xml::Reader;
use quick_xml::events::Event;
use serde_json::{Map, Value as JsonValue};

/// 将 XML 文本转换为 `{ tag, attributes, children, text }` 结构。
pub fn parse_xml_to_json(text: &str) -> Result<JsonValue, ()> {
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
        .map(|attribute| {
            (
                String::from_utf8_lossy(attribute.key.as_ref()).to_string(),
                String::from_utf8_lossy(attribute.value.as_ref()).to_string(),
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
