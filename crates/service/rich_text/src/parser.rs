use super::params::RichTextParams;
use super::{RichText, RichTextSegment, TextStyle, parse_text_color};
use tg_core_input::format_key_display;

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
pub(super) fn parse_auto(text: &str, params: Option<&RichTextParams>) -> RichText {
  text.strip_prefix(RICH_TEXT_PREFIX).map_or_else(
    || plain_text(text),
    |body| parse_formatted_text(body, params),
  )
}

pub(super) fn parse_plain(text: &str) -> RichText {
  plain_text(text)
}

pub(super) fn parse_rich(text: &str, params: Option<&RichTextParams>) -> RichText {
  parse_formatted_text(text, params)
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
      "key_default" => resolve_default_key(key, params),
      _ => None,
    }
  } else {
    None
  }
}

fn resolve_value(key: &str, params: Option<&RichTextParams>) -> Option<String> {
  params.and_then(|p| p.values.get(key)).cloned()
}

fn resolve_key(action: &str, params: Option<&RichTextParams>) -> Option<String> {
  let patterns = params?.key_actions.get(action)?;
  Some(format_key_parameter(patterns))
}

fn resolve_default_key(action: &str, params: Option<&RichTextParams>) -> Option<String> {
  let patterns = params?.key_default_actions.get(action)?;
  Some(format_key_parameter(patterns))
}

fn format_key_parameter(patterns: &[Vec<String>]) -> String {
  if patterns.is_empty() {
    "[]".to_string()
  } else {
    format_key_display(patterns)
  }
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
    if color_value.trim() == "reverse" {
      current_style.reverse_foreground();
      return true;
    }
    if let Some(color) = parse_text_color(color_value) {
      current_style.set_foreground(color);
      return true;
    }
    return false;
  }

  if let Some(color_value) = tag.strip_prefix("bg:") {
    if color_value.trim() == "reverse" {
      current_style.reverse_background();
      return true;
    }
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
  use crate::{TerminalColor, TextColor};
  use std::collections::HashMap;

  fn make_params(
    values: HashMap<String, String>,
    key_actions: HashMap<String, Vec<Vec<String>>>,
  ) -> RichTextParams {
    RichTextParams {
      values,
      key_default_actions: key_actions.clone(),
      key_actions,
    }
  }

  #[test]
  fn key_param_single() {
    let mut ka = HashMap::new();
    ka.insert("jump".to_string(), vec![vec!["shift".to_string()]]);
    let params = RichTextParams::from_key_actions(&ka);
    let rt = parse_auto("f%{key:jump}", Some(&params));
    assert_eq!(rt.segments[0].text, "[Shift]");
  }

  #[test]
  fn key_and_key_default_use_separate_maps() {
    let user = HashMap::from([("jump".to_string(), vec![vec!["j".to_string()]])]);
    let defaults = HashMap::from([("jump".to_string(), vec![vec!["space".to_string()]])]);
    let params = RichTextParams::from_key_action_maps(&user, &defaults);
    let rt = parse_auto("f%{key:jump}/{key_default:jump}", Some(&params));
    assert_eq!(rt.segments[0].text, "[J]/[Space]");
  }

  #[test]
  fn empty_key_parameter_is_visible() {
    let keys = HashMap::from([("optional".to_string(), Vec::new())]);
    let params = RichTextParams::from_key_actions(&keys);
    let rt = parse_auto("f%{key:optional}/{key_default:optional}", Some(&params));
    assert_eq!(rt.segments[0].text, "[]/[]");
  }

  #[test]
  fn package_key_params_do_not_resolve_values() {
    let mut ka = HashMap::new();
    ka.insert("move_up".to_string(), vec![vec!["w".to_string()]]);
    let params = RichTextParams::from_key_actions(&ka);
    let rt = parse_auto("f%{key:move_up} {value:name}", Some(&params));
    assert_eq!(rt.segments[0].text, "[W] {value:name}");
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
    let rt = parse_auto("f%{key:move}", Some(&params));
    assert_eq!(rt.segments[0].text, "[D]/[Shift + ←]");
  }

  #[test]
  fn value_param() {
    let mut values = HashMap::new();
    values.insert("name".to_string(), "Alice".to_string());
    let params = make_params(values, HashMap::new());
    let rt = parse_auto("f%{value:name}", Some(&params));
    assert_eq!(rt.segments[0].text, "Alice");
  }

  #[test]
  fn value_parameters_require_an_explicit_namespace() {
    let mut values = HashMap::new();
    values.insert("name".to_string(), "Bob".to_string());
    let params = make_params(values, HashMap::new());
    let rt = parse_auto("f%{name}", Some(&params));
    assert_eq!(rt.segments[0].text, "{name}");
  }

  #[test]
  fn key_not_found_keeps_original() {
    let params = make_params(HashMap::new(), HashMap::new());
    let rt = parse_auto("f%{key:unknown}", Some(&params));
    assert_eq!(rt.segments[0].text, "{key:unknown}");
  }

  #[test]
  fn mixed_value_and_key() {
    let mut values = HashMap::new();
    values.insert("action".to_string(), "Jump".to_string());
    let mut ka = HashMap::new();
    ka.insert("jump".to_string(), vec![vec!["space".to_string()]]);
    let params = make_params(values, ka);
    let rt = parse_auto("f%{value:action}: press {key:jump}", Some(&params));
    assert_eq!(rt.segments[0].text, "Jump: press [Space]");
  }

  #[test]
  fn reverse_explicit_rgb_foreground_and_background() {
    let rt = parse_auto(
      "f%<fg:#102030><bg:rgb(1,2,3)>A<fg:reverse><bg:reverse>B",
      None,
    );

    assert_eq!(rt.segments.len(), 2);
    assert_eq!(
      rt.segments[0].style.foreground,
      Some(TextColor::Rgb {
        r: 0x10,
        g: 0x20,
        b: 0x30,
      })
    );
    assert_eq!(
      rt.segments[1].style.foreground,
      Some(TextColor::Rgb {
        r: 0xef,
        g: 0xdf,
        b: 0xcf,
      })
    );
    assert_eq!(
      rt.segments[1].style.background,
      Some(TextColor::Rgb {
        r: 254,
        g: 253,
        b: 252,
      })
    );
  }

  #[test]
  fn reverse_keeps_terminal_colors_and_is_not_rendered() {
    let rt = parse_auto("f%<fg:red>A<fg:reverse>B<bg:reverse>C", None);

    assert_eq!(rt.segments.len(), 3);
    assert_eq!(
      rt.segments[2].style.foreground,
      Some(TextColor::Terminal(TerminalColor::Red))
    );
    assert_eq!(rt.segments[2].style.background, None);
    assert_eq!(
      rt.segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<String>(),
      "ABC"
    );
  }

  #[test]
  fn semantic_color_aliases_apply_to_foreground_and_background() {
    let rt = parse_auto("f%<fg:gray>A<fg:bright_gray>B<fg:white><bg:grey>C", None);

    assert_eq!(
      rt.segments[0].style.foreground,
      Some(TextColor::Terminal(TerminalColor::BrightBlack))
    );
    assert_eq!(
      rt.segments[1].style.foreground,
      Some(TextColor::Terminal(TerminalColor::White))
    );
    assert_eq!(
      rt.segments[2].style.foreground,
      Some(TextColor::Terminal(TerminalColor::BrightWhite))
    );
    assert_eq!(
      rt.segments[2].style.background,
      Some(TextColor::Terminal(TerminalColor::BrightBlack))
    );
  }
}
