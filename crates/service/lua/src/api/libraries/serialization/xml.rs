use std::collections::{BTreeMap, HashSet};

use mlua::{Lua, MultiValue, Table, Value};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use super::{args, value};

const MAX_XML_DEPTH: usize = 32;
const MAX_XML_NODES: usize = 16_384;

#[derive(Debug)]
struct Node {
  name: String,
  attributes: Vec<(String, String)>,
  text: String,
  children: Vec<Node>,
}

pub(super) fn install(lua: &Lua, source: &Table) -> mlua::Result<()> {
  source.raw_set(
    "xml_encode",
    lua.create_function(|_, values: MultiValue| {
      let method = "serialization.xml_encode";
      let root_value = args::one(method, "value", values)?;
      let Value::Table(root) = root_value else {
        return Err(args::invalid(method, "value", "table", &root_value));
      };
      let (name, root_value) = root_entry(root, method)?;
      let mut seen = HashSet::new();
      let mut node_count = 0;
      let node = lua_element(name, root_value, method, 1, &mut node_count, &mut seen)?;
      let mut output = String::new();
      encode_node(&node, &mut output, method)?;
      value::bounded_text(method, output)
    })?,
  )?;
  source.raw_set(
    "xml_decode",
    lua.create_function(|lua, values: MultiValue| {
      let method = "serialization.xml_decode";
      let text = value::text_argument(values, method)?;
      if text.contains("<!DOCTYPE") || text.contains("<!ENTITY") {
        return Err(args::message(
          method,
          "DTD and entity declarations are not supported",
        ));
      }
      let node = parse(&text, method)?;
      let result = lua.create_table()?;
      let name = node.name.clone();
      result.raw_set(name, node_to_lua(lua, node, method, 1)?)?;
      Ok(result)
    })?,
  )
}

fn root_entry(root: Table, method: &str) -> mlua::Result<(String, Value)> {
  let mut entry = None;
  for pair in root.pairs::<Value, Value>() {
    let (key, value) = pair?;
    let Value::String(key) = key else {
      return Err(args::message(
        method,
        "XML root table must contain exactly one named element",
      ));
    };
    let key = key
      .to_str()
      .map_err(|_| args::message(method, "XML element names must be UTF-8"))?
      .to_string();
    validate_name(&key, "element", method)?;
    if entry.replace((key, value)).is_some() {
      return Err(args::message(
        method,
        "XML root table must contain exactly one named element",
      ));
    }
  }
  entry.ok_or_else(|| {
    args::message(
      method,
      "XML root table must contain exactly one named element",
    )
  })
}

fn lua_element(
  name: String,
  value: Value,
  method: &str,
  depth: usize,
  node_count: &mut usize,
  seen: &mut HashSet<usize>,
) -> mlua::Result<Node> {
  if depth > MAX_XML_DEPTH {
    return Err(args::message(method, "XML value exceeds 32 levels"));
  }
  *node_count += 1;
  if *node_count > MAX_XML_NODES {
    return Err(args::message(method, "XML value exceeds 16384 nodes"));
  }

  let Value::Table(table) = value else {
    return Ok(Node {
      name,
      attributes: Vec::new(),
      text: scalar_text(value, method)?,
      children: Vec::new(),
    });
  };

  let pointer = table.to_pointer() as usize;
  if !seen.insert(pointer) {
    return Err(args::message(method, "cyclic tables are not serializable"));
  }
  let result = lua_table_element(name, table, method, depth, node_count, seen);
  seen.remove(&pointer);
  result
}

fn lua_table_element(
  name: String,
  table: Table,
  method: &str,
  depth: usize,
  node_count: &mut usize,
  seen: &mut HashSet<usize>,
) -> mlua::Result<Node> {
  let mut attributes = Vec::new();
  let mut explicit_text = None;
  let mut positional = BTreeMap::new();
  let mut named_children = Vec::new();

  for pair in table.pairs::<Value, Value>() {
    let (key, value) = pair?;
    match key {
      Value::Integer(index) if index > 0 => {
        positional.insert(index as usize, value);
      }
      Value::String(key) => {
        let key = key
          .to_str()
          .map_err(|_| args::message(method, "XML element names must be UTF-8"))?
          .to_string();
        match key.as_str() {
          "_attr" => attributes = lua_attributes(value, method)?,
          "_text" => explicit_text = Some(scalar_text(value, method)?),
          _ if key.starts_with('_') => {
            return Err(args::message(
              method,
              format!("unsupported reserved XML field '{key}'"),
            ));
          }
          _ => {
            validate_name(&key, "element", method)?;
            named_children.push((key, value));
          }
        }
      }
      _ => {
        return Err(args::message(
          method,
          "XML tables only support positive integer and string keys",
        ));
      }
    }
  }

  let mut positional_text = String::new();
  for expected in 1..=positional.len() {
    let item = positional
      .remove(&expected)
      .ok_or_else(|| args::message(method, "XML positional text values must be contiguous"))?;
    positional_text.push_str(&scalar_text(item, method)?);
  }
  if explicit_text.is_some() && !positional_text.is_empty() {
    return Err(args::message(
      method,
      "XML element cannot contain both _text and positional text",
    ));
  }
  let text = explicit_text.unwrap_or(positional_text);
  if !text.is_empty() && !named_children.is_empty() {
    return Err(args::message(
      method,
      "mixed XML text and child elements are not supported",
    ));
  }

  let mut children = Vec::new();
  for (child_name, child_value) in named_children {
    if let Value::Table(child_table) = &child_value
      && is_pure_sequence(child_table.clone())?
    {
      for child in child_table.clone().sequence_values::<Value>() {
        children.push(lua_element(
          child_name.clone(),
          child?,
          method,
          depth + 1,
          node_count,
          seen,
        )?);
      }
      continue;
    }
    children.push(lua_element(
      child_name,
      child_value,
      method,
      depth + 1,
      node_count,
      seen,
    )?);
  }

  Ok(Node {
    name,
    attributes,
    text,
    children,
  })
}

fn lua_attributes(value: Value, method: &str) -> mlua::Result<Vec<(String, String)>> {
  let Value::Table(table) = value else {
    return Err(args::invalid(method, "_attr", "table", &value));
  };
  let mut attributes = Vec::new();
  for pair in table.pairs::<Value, Value>() {
    let (key, value) = pair?;
    let Value::String(key) = key else {
      return Err(args::message(method, "XML attribute names must be strings"));
    };
    let key = key
      .to_str()
      .map_err(|_| args::message(method, "XML attribute names must be UTF-8"))?
      .to_string();
    validate_name(&key, "attribute", method)?;
    attributes.push((key, scalar_text(value, method)?));
  }
  attributes.sort_by(|left, right| left.0.cmp(&right.0));
  Ok(attributes)
}

fn scalar_text(value: Value, method: &str) -> mlua::Result<String> {
  match value {
    Value::Nil => Ok(String::new()),
    Value::Boolean(value) => Ok(value.to_string()),
    Value::Integer(value) => Ok(value.to_string()),
    Value::Number(value) if value.is_finite() => Ok(value.to_string()),
    Value::String(value) => value
      .to_str()
      .map(|value| value.to_string())
      .map_err(|_| args::message(method, "XML text must be valid UTF-8")),
    other => Err(args::message(
      method,
      format!(
        "XML text and attributes must be scalar values, got {}",
        other.type_name()
      ),
    )),
  }
}

fn is_pure_sequence(table: Table) -> mlua::Result<bool> {
  let mut count = 0_usize;
  let mut largest = 0_usize;
  for pair in table.pairs::<Value, Value>() {
    let (key, _) = pair?;
    let Value::Integer(index) = key else {
      return Ok(false);
    };
    if index < 1 {
      return Ok(false);
    }
    count += 1;
    largest = largest.max(index as usize);
  }
  Ok(count > 0 && count == largest)
}

fn encode_node(node: &Node, output: &mut String, method: &str) -> mlua::Result<()> {
  output.push('<');
  output.push_str(&node.name);
  for (name, value) in &node.attributes {
    output.push(' ');
    output.push_str(name);
    output.push_str("=\"");
    escape(value, output);
    output.push('"');
  }
  if node.text.is_empty() && node.children.is_empty() {
    output.push_str("/>");
  } else {
    output.push('>');
    escape(&node.text, output);
    for child in &node.children {
      encode_node(child, output, method)?;
    }
    output.push_str("</");
    output.push_str(&node.name);
    output.push('>');
  }
  if output.len() > args::MAX_API_STRING_BYTES {
    return Err(args::message(method, "serialized output exceeds 1 MiB"));
  }
  Ok(())
}

fn parse(text: &str, method: &str) -> mlua::Result<Node> {
  let mut reader = Reader::from_str(text);
  reader.config_mut().trim_text(true);
  let mut stack: Vec<Node> = Vec::new();
  let mut root = None;
  let mut node_count = 0_usize;
  loop {
    match reader.read_event() {
      Ok(Event::Start(event)) => {
        if stack.len() >= MAX_XML_DEPTH {
          return Err(args::message(method, "XML value exceeds 32 levels"));
        }
        node_count += 1;
        if node_count > MAX_XML_NODES {
          return Err(args::message(method, "XML value exceeds 16384 nodes"));
        }
        stack.push(start_node(&event, reader.decoder(), method)?);
      }
      Ok(Event::Empty(event)) => {
        node_count += 1;
        if node_count > MAX_XML_NODES {
          return Err(args::message(method, "XML value exceeds 16384 nodes"));
        }
        let node = start_node(&event, reader.decoder(), method)?;
        append_node(node, &mut stack, &mut root, method)?;
      }
      Ok(Event::Text(event)) => {
        let decoded = event
          .decode()
          .map_err(|_| args::message(method, "invalid XML text"))?;
        let decoded = quick_xml::escape::unescape(&decoded)
          .map_err(|_| args::message(method, "invalid XML entity"))?;
        if let Some(node) = stack.last_mut() {
          node.text.push_str(&decoded);
        } else if !decoded.trim().is_empty() {
          return Err(args::message(
            method,
            "text outside the XML root is not supported",
          ));
        }
      }
      Ok(Event::CData(event)) => {
        let decoded = event
          .decode()
          .map_err(|_| args::message(method, "invalid XML CDATA"))?;
        if let Some(node) = stack.last_mut() {
          node.text.push_str(&decoded);
        }
      }
      Ok(Event::End(event)) => {
        let node = stack
          .pop()
          .ok_or_else(|| args::message(method, "unbalanced XML element"))?;
        if event.name().as_ref() != node.name.as_bytes() {
          return Err(args::message(method, "mismatched XML closing element"));
        }
        if !node.text.trim().is_empty() && !node.children.is_empty() {
          return Err(args::message(
            method,
            "mixed XML text and child elements are not supported",
          ));
        }
        append_node(node, &mut stack, &mut root, method)?;
      }
      Ok(Event::Decl(_)) | Ok(Event::Comment(_)) | Ok(Event::PI(_)) => {}
      Ok(Event::DocType(_)) | Ok(Event::GeneralRef(_)) => {
        return Err(args::message(
          method,
          "DTD and custom entities are not supported",
        ));
      }
      Ok(Event::Eof) => break,
      Err(_) => return Err(args::message(method, "invalid XML data")),
    }
  }
  if !stack.is_empty() {
    return Err(args::message(method, "unclosed XML element"));
  }
  root.ok_or_else(|| args::message(method, "XML document has no root element"))
}

fn start_node(
  event: &BytesStart<'_>,
  decoder: quick_xml::encoding::Decoder,
  method: &str,
) -> mlua::Result<Node> {
  let name = std::str::from_utf8(event.name().as_ref())
    .map_err(|_| args::message(method, "XML element names must be UTF-8"))?
    .to_string();
  validate_name(&name, "element", method)?;
  let mut attributes = Vec::new();
  for attribute in event.attributes().with_checks(true) {
    let attribute = attribute.map_err(|_| args::message(method, "invalid XML attribute"))?;
    let key = std::str::from_utf8(attribute.key.as_ref())
      .map_err(|_| args::message(method, "XML attribute names must be UTF-8"))?
      .to_string();
    validate_name(&key, "attribute", method)?;
    let value = attribute
      .decoded_and_normalized_value(quick_xml::XmlVersion::Implicit1_0, decoder)
      .map_err(|_| args::message(method, "invalid XML attribute value"))?
      .to_string();
    attributes.push((key, value));
  }
  Ok(Node {
    name,
    attributes,
    text: String::new(),
    children: Vec::new(),
  })
}

fn append_node(
  node: Node,
  stack: &mut [Node],
  root: &mut Option<Node>,
  method: &str,
) -> mlua::Result<()> {
  if let Some(parent) = stack.last_mut() {
    if !parent.text.trim().is_empty() {
      return Err(args::message(
        method,
        "mixed XML text and child elements are not supported",
      ));
    }
    parent.children.push(node);
  } else if root.replace(node).is_some() {
    return Err(args::message(
      method,
      "XML document has multiple root elements",
    ));
  }
  Ok(())
}

fn node_to_lua(lua: &Lua, node: Node, method: &str, depth: usize) -> mlua::Result<Value> {
  if depth > MAX_XML_DEPTH {
    return Err(args::message(method, "XML value exceeds 32 levels"));
  }
  if node.attributes.is_empty() && node.children.is_empty() {
    return Ok(Value::String(lua.create_string(node.text)?));
  }

  let result = lua.create_table()?;
  if !node.attributes.is_empty() {
    let attributes = lua.create_table()?;
    for (name, value) in node.attributes {
      attributes.raw_set(name, value)?;
    }
    result.raw_set("_attr", attributes)?;
  }
  if !node.text.is_empty() {
    result.raw_set("_text", node.text)?;
  }

  let mut groups: Vec<(String, Vec<Node>)> = Vec::new();
  for child in node.children {
    if let Some((_, nodes)) = groups.iter_mut().find(|(name, _)| *name == child.name) {
      nodes.push(child);
    } else {
      groups.push((child.name.clone(), vec![child]));
    }
  }
  for (name, mut nodes) in groups {
    if nodes.len() == 1 {
      result.raw_set(name, node_to_lua(lua, nodes.remove(0), method, depth + 1)?)?;
    } else {
      let values = lua.create_table()?;
      for (index, child) in nodes.into_iter().enumerate() {
        values.raw_set(index + 1, node_to_lua(lua, child, method, depth + 1)?)?;
      }
      result.raw_set(name, values)?;
    }
  }
  Ok(Value::Table(result))
}

fn validate_name(name: &str, kind: &str, method: &str) -> mlua::Result<()> {
  let mut chars = name.chars();
  let valid_start = chars
    .next()
    .is_some_and(|ch| ch == '_' || ch.is_alphabetic());
  if !valid_start || chars.any(|ch| !(ch == '_' || ch == '-' || ch == '.' || ch.is_alphanumeric()))
  {
    Err(args::message(
      method,
      format!("invalid XML {kind} name '{name}'"),
    ))
  } else {
    Ok(())
  }
}

fn escape(value: &str, output: &mut String) {
  for ch in value.chars() {
    match ch {
      '&' => output.push_str("&amp;"),
      '<' => output.push_str("&lt;"),
      '>' => output.push_str("&gt;"),
      '"' => output.push_str("&quot;"),
      '\'' => output.push_str("&apos;"),
      ch => output.push(ch),
    }
  }
}
