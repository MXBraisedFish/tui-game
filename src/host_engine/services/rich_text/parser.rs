use super::params::RichTextParams;
use super::{RichText, RichTextSegment, TextStyle, parse_text_color};
use crate::host_engine::services::input::format_key_display;

const RICH_TEXT_PREFIX: &str = "f%";

enum ParameterReadResult {
  Closed(String),
  Broken(String),
}

enum TagReadResult {
  Closed(String),
  Broken(String),
}

/// 解析富文本字符串，将 `<tag>` 标签转换为样式段、`{param}` 替换为实际值。
pub fn parse(text: &str, params: Option<&RichTextParams>) -> RichText {
  let body = if text.starts_with(RICH_TEXT_PREFIX) {

    &text[RICH_TEXT_PREFIX.len()..]
  } else if params.is_some() {

    return parse_formatted_text(text, params);
  } else {

    return plain_text(text);
  };

  parse_formatted_text(body, params)
}

fn plain_text(text: &str) -> RichText {
  RichText {
    segments: vec![RichTextSegment {
      text: text.to_string(),
      style: TextStyle::default(),
    }],
  }
}

fn parse_formatted_text(text: &str, params: Option<&RichTextParams>) -> RichText {

  let mut segments = Vec::new();

  let mut output = String::new();

  let mut current_style = TextStyle::default();

  let mut chars = text.chars().peekable();

  while let Some(ch) = chars.next() {

    if ch == '\\' {
      if let Some(escaped) = read_escaped_char(&mut chars) {
        output.push(escaped);
      } else {
        output.push(ch);
      }
      continue;
    }

    if ch == '{' {
      match read_parameter_name(&mut chars) {
        ParameterReadResult::Closed(name) => {
          write_resolved_parameter(&mut output, &name, params);
        }
        ParameterReadResult::Broken(content) => {
          output.push('{');
          output.push_str(&content);
        }
      }
      continue;
    }

    if ch == '<' {
      match read_tag(&mut chars) {
        TagReadResult::Closed(tag) => {
          flush_segment(&mut segments, &mut output, &current_style);

          if !apply_tag(&tag, &mut current_style) {

            output.push('<');
            output.push_str(&tag);
            output.push('>');
          }
        }
        TagReadResult::Broken(content) => {
          output.push('<');
          output.push_str(&content);
        }
      }
      continue;
    }

    output.push(ch);
  }

  flush_segment(&mut segments, &mut output, &current_style);

  RichText { segments }
}

fn read_parameter_name<I>(chars: &mut std::iter::Peekable<I>) -> ParameterReadResult
where
  I: Iterator<Item = char>,
{
  let mut name = String::new();

  while let Some(next) = chars.peek() {

    if *next == '{' {
      return ParameterReadResult::Broken(name);
    }

    let ch = chars.next().unwrap();

    if ch == '\\' {
      if let Some(escaped) = read_escaped_char(chars) {
        name.push(escaped);
      } else {
        name.push(ch);
      }
      continue;
    }

    if ch == '}' {
      return ParameterReadResult::Closed(name);
    }

    name.push(ch);
  }

  ParameterReadResult::Broken(name)
}

fn write_resolved_parameter(output: &mut String, name: &str, params: Option<&RichTextParams>) {
  if name.is_empty() {
    output.push_str("{}");
    return;
  }

  if let Some(value) = resolve_parameter(name, params) {
    output.push_str(&value);
  } else {
    output.push('{');
    output.push_str(name);
    output.push('}');
  }
}

fn resolve_parameter(name: &str, params: Option<&RichTextParams>) -> Option<String> {
  if let Some((ns, key)) = name.split_once(':') {
    match ns {
      "value" => resolve_value(key, params),
      "key" => resolve_key(key, params),
      _ => None,
    }
  } else {

    resolve_value(name, params)
  }
}

fn resolve_value(key: &str, params: Option<&RichTextParams>) -> Option<String> {
  params.and_then(|p| p.values.get(key)).cloned()
}

fn resolve_key(action: &str, params: Option<&RichTextParams>) -> Option<String> {
  let patterns = params?.key_actions.get(action)?;
  Some(format_key_display(patterns))
}

fn read_tag<I>(chars: &mut std::iter::Peekable<I>) -> TagReadResult
where
  I: Iterator<Item = char>,
{

  let mut tag = String::new();

  while let Some(next) = chars.peek() {

    if *next == '<' {
      return TagReadResult::Broken(tag);
    }

    let ch = chars.next().unwrap();

    if ch == '\\' {
      if let Some(escaped) = read_escaped_char(chars) {
        tag.push(escaped);
      } else {
        tag.push(ch);
      }
      continue;
    }

    if ch == '>' {
      return TagReadResult::Closed(tag);
    }

    tag.push(ch);
  }

  TagReadResult::Broken(tag)
}

fn read_escaped_char<I>(chars: &mut std::iter::Peekable<I>) -> Option<char>
where
  I: Iterator<Item = char>,
{
  let next = chars.peek()?;
  match next {
    '{' | '}' | '<' | '>' | '\\' => chars.next(),
    _ => None,
  }
}

fn flush_segment(segments: &mut Vec<RichTextSegment>, output: &mut String, style: &TextStyle) {
  if output.is_empty() {
    return;
  }
  segments.push(RichTextSegment {
    text: std::mem::take(output),
    style: style.clone(),
  });
}

fn apply_tag(tag: &str, current_style: &mut TextStyle) -> bool {

  let tag = tag.trim();

  if tag == "reset" {
    current_style.reset();
    return true;
  }

  if tag == "/fg" {
    current_style.clear_foreground();
    return true;
  }

  if tag == "/bg" {
    current_style.clear_background();
    return true;
  }

  if let Some(style_name) = tag.strip_prefix('/') {
    return current_style.disable_style(style_name.trim());
  }

  if let Some(color_value) = tag.strip_prefix("fg:") {
    if let Some(color) = parse_text_color(color_value) {
      current_style.set_foreground(color);
      return true;
    }
    return false;
  }

  if let Some(color_value) = tag.strip_prefix("bg:") {
    if let Some(color) = parse_text_color(color_value) {
      current_style.set_background(color);
      return true;
    }
    return false;
  }

  current_style.enable_style(tag)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;

  fn make_params(
    values: HashMap<String, String>,
    key_actions: HashMap<String, Vec<Vec<String>>>,
  ) -> RichTextParams {
    RichTextParams {
      values,
      key_actions,
    }
  }

  #[test]
  fn key_param_single() {
    let mut ka = HashMap::new();
    ka.insert("jump".to_string(), vec![vec!["shift".to_string()]]);
    let params = make_params(HashMap::new(), ka);
    let rt = parse("f%{key:jump}", Some(&params));
    assert_eq!(rt.segments[0].text, "[Shift]");
  }

  #[test]
  fn key_param_multi_pattern() {
    let mut ka = HashMap::new();
    ka.insert(
      "move".to_string(),
      vec![
        vec!["d".to_string()],
        vec!["left".to_string(), "shift".to_string()],
      ],
    );
    let params = make_params(HashMap::new(), ka);
    let rt = parse("f%{key:move}", Some(&params));
    assert_eq!(rt.segments[0].text, "[D]/[Shift + ←]");
  }

  #[test]
  fn value_param() {
    let mut values = HashMap::new();
    values.insert("name".to_string(), "Alice".to_string());
    let params = make_params(values, HashMap::new());
    let rt = parse("f%{value:name}", Some(&params));
    assert_eq!(rt.segments[0].text, "Alice");
  }

  #[test]
  fn backward_compat_no_prefix() {
    let mut values = HashMap::new();
    values.insert("name".to_string(), "Bob".to_string());
    let params = make_params(values, HashMap::new());
    let rt = parse("f%{name}", Some(&params));
    assert_eq!(rt.segments[0].text, "Bob");
  }

  #[test]
  fn key_not_found_keeps_original() {
    let params = make_params(HashMap::new(), HashMap::new());
    let rt = parse("f%{key:unknown}", Some(&params));
    assert_eq!(rt.segments[0].text, "{key:unknown}");
  }

  #[test]
  fn mixed_value_and_key() {
    let mut values = HashMap::new();
    values.insert("action".to_string(), "Jump".to_string());
    let mut ka = HashMap::new();
    ka.insert("jump".to_string(), vec![vec!["space".to_string()]]);
    let params = make_params(values, ka);
    let rt = parse("f%{value:action}: press {key:jump}", Some(&params));
    assert_eq!(rt.segments[0].text, "Jump: press [Space]");
  }
}
