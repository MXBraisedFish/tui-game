//! XML 序列化

use serde_json::Value as JsonValue;

/// 将 JSON 中间值序列化为 XML 字符串。
pub fn serialize_xml(value: &JsonValue) -> String {
    let mut output = String::new();
    value_to_xml("root", value, &mut output);
    output
}

fn value_to_xml(tag: &str, value: &JsonValue, output: &mut String) {
    match value {
        JsonValue::Null => {
            output.push('<');
            output.push_str(tag);
            output.push_str("/>");
        }
        JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => {
            output.push('<');
            output.push_str(tag);
            output.push('>');
            output.push_str(&escape_xml(json_scalar_to_string(value).as_str()));
            output.push_str("</");
            output.push_str(tag);
            output.push('>');
        }
        JsonValue::Array(items) => {
            output.push('<');
            output.push_str(tag);
            output.push('>');
            for item in items {
                value_to_xml("item", item, output);
            }
            output.push_str("</");
            output.push_str(tag);
            output.push('>');
        }
        JsonValue::Object(values) => {
            output.push('<');
            output.push_str(tag);
            output.push('>');
            for (key, item) in values {
                value_to_xml(sanitize_xml_tag(key).as_str(), item, output);
            }
            output.push_str("</");
            output.push_str(tag);
            output.push('>');
        }
    }
}

fn sanitize_xml_tag(tag: &str) -> String {
    if tag.is_empty() {
        return "item".to_string();
    }

    let mut output = String::new();
    for (index, character) in tag.chars().enumerate() {
        let valid = character.is_ascii_alphanumeric() || character == '_' || character == '-';
        if index == 0 {
            if character.is_ascii_alphabetic() || character == '_' {
                output.push(character);
            } else if valid {
                output.push('_');
                output.push(character);
            } else {
                output.push('_');
            }
        } else if valid {
            output.push(character);
        } else {
            output.push('_');
        }
    }

    if output.is_empty() {
        "item".to_string()
    } else {
        output
    }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn json_scalar_to_string(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => String::new(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        JsonValue::String(value) => value.clone(),
        JsonValue::Array(_) | JsonValue::Object(_) => {
            serde_json::to_string(value).unwrap_or_default()
        }
    }
}
