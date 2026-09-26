use std::collections::BTreeMap;

use mlua::{Lua, MultiValue, Table};

use super::{args, value};

pub(super) fn install(lua: &Lua, source: &Table) -> mlua::Result<()> {
  source.raw_set(
    "ini_encode",
    lua.create_function(|_, values: MultiValue| {
      let method = "serialization.ini_encode";
      let data = value::lua_to_json(args::one(method, "t", values)?, method)?;
      let serde_json::Value::Object(entries) = data else {
        return Err(args::message(method, "INI root must be an object table"));
      };
      let mut globals = BTreeMap::new();
      let mut sections = BTreeMap::new();
      for (key, value) in entries {
        validate_name(&key, method)?;
        match value {
          serde_json::Value::Object(values) => {
            let mut section = BTreeMap::new();
            for (child, value) in values {
              validate_name(&child, method)?;
              section.insert(child, scalar(value, method)?);
            }
            sections.insert(key, section);
          }
          value => {
            globals.insert(key, scalar(value, method)?);
          }
        }
      }
      let mut output = String::new();
      for (key, value) in globals {
        output.push_str(&key);
        output.push('=');
        output.push_str(&value);
        output.push('\n');
      }
      for (section, values) in sections {
        if !output.is_empty() && !output.ends_with("\n\n") {
          output.push('\n');
        }
        output.push('[');
        output.push_str(&section);
        output.push_str("]\n");
        for (key, value) in values {
          output.push_str(&key);
          output.push('=');
          output.push_str(&value);
          output.push('\n');
        }
      }
      value::bounded_text(method, output)
    })?,
  )?;
  source.raw_set(
    "ini_decode",
    lua.create_function(|lua, values: MultiValue| {
      let method = "serialization.ini_decode";
      let text = value::text_argument(values, method)?;
      let root = lua.create_table()?;
      let mut current: Option<Table> = None;
      for (line_index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
          continue;
        }
        if line.starts_with('[') {
          let Some(name) = line
            .strip_prefix('[')
            .and_then(|line| line.strip_suffix(']'))
          else {
            return Err(args::message(
              method,
              format!("invalid section on line {}", line_index + 1),
            ));
          };
          validate_name(name.trim(), method)?;
          let section = lua.create_table()?;
          root.raw_set(name.trim(), section.clone())?;
          current = Some(section);
          continue;
        }
        let Some((key, value)) = line.split_once('=') else {
          return Err(args::message(
            method,
            format!("invalid entry on line {}", line_index + 1),
          ));
        };
        let key = key.trim();
        validate_name(key, method)?;
        let target = current.as_ref().unwrap_or(&root);
        if target.contains_key(key)? {
          return Err(args::message(method, format!("duplicate key '{key}'")));
        }
        target.raw_set(key, value.trim())?;
      }
      Ok(root)
    })?,
  )
}

fn validate_name(value: &str, method: &str) -> mlua::Result<()> {
  if value.is_empty()
    || value
      .chars()
      .any(|ch| matches!(ch, '\n' | '\r' | '[' | ']' | '=' | ';' | '#'))
  {
    Err(args::message(
      method,
      "INI keys and section names contain invalid characters",
    ))
  } else {
    Ok(())
  }
}

fn scalar(value: serde_json::Value, method: &str) -> mlua::Result<String> {
  match value {
    serde_json::Value::Null => Ok(String::new()),
    serde_json::Value::Bool(value) => Ok(value.to_string()),
    serde_json::Value::Number(value) => Ok(value.to_string()),
    serde_json::Value::String(value) if !value.contains(['\n', '\r']) => Ok(value),
    serde_json::Value::String(_) => {
      Err(args::message(method, "INI values cannot contain newlines"))
    }
    _ => Err(args::message(
      method,
      "INI supports only one section level and scalar values",
    )),
  }
}
