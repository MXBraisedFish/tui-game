//! 包内翻译动态变量替换

use std::collections::HashMap;

use mlua::{Table, Value};

/// 从 Lua 表读取翻译动态变量。
pub fn read_parameter_table(table: &Table) -> mlua::Result<HashMap<String, String>> {
    let mut parameters = HashMap::new();

    for pair in table.clone().pairs::<Value, Value>() {
        let (key, value) = pair?;
        let Value::String(key) = key else {
            continue;
        };

        parameters.insert(
            key.to_str()?.to_string(),
            parameter_value_to_string(&value)?,
        );
    }

    Ok(parameters)
}

/// 替换文本中的 `{name}` 动态变量。
///
/// 未声明的变量保持原样，避免破坏普通文本和富文本指令。
pub fn apply_parameters(text: &str, parameters: &HashMap<String, String>) -> String {
    if parameters.is_empty() {
        return text.to_string();
    }

    let mut output = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();

    while let Some((index, character)) = chars.next() {
        if character != '{' {
            output.push(character);
            continue;
        }

        let placeholder_start = index;
        let mut placeholder_name = String::new();
        let mut placeholder_end = None;

        while let Some((inner_index, inner_character)) = chars.next() {
            if inner_character == '}' {
                placeholder_end = Some(inner_index + inner_character.len_utf8());
                break;
            }
            if inner_character == '{' {
                break;
            }
            placeholder_name.push(inner_character);
        }

        if let Some(end_index) = placeholder_end {
            if let Some(replacement) = parameters.get(placeholder_name.as_str()) {
                output.push_str(replacement);
            } else {
                output.push_str(&text[placeholder_start..end_index]);
            }
        } else {
            output.push_str(&text[placeholder_start..]);
            break;
        }
    }

    output
}

fn parameter_value_to_string(value: &Value) -> mlua::Result<String> {
    match value {
        Value::Nil => Ok("nil".to_string()),
        Value::Boolean(value) => Ok(value.to_string()),
        Value::Integer(value) => Ok(value.to_string()),
        Value::Number(value) => Ok(value.to_string()),
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(mlua::Error::external(
            "translation parameter value must be a scalar",
        )),
    }
}
