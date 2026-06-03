use super::RichTextParams;
use super::{RichText, RichTextSegment, TextStyle, parse_text_color};

// 富文本前缀
const RICH_TEXT_PREFIX: &str = "f%";

// 参数读取结果
enum ParameterReadResult {
  Closed(String), // 关闭
  Broken(String), // 损坏
}

// 标签读取结果
enum TagReadResult {
  Closed(String),
  Broken(String),
}

// 解析入口
pub fn parse(text: &str, params: Option<&RichTextParams>) -> RichText {
  // 前缀判断
  if !text.starts_with(RICH_TEXT_PREFIX) {
    return plain_text(text);
  }

  // 进行富文本解析
  parse_formatted_text(&text[RICH_TEXT_PREFIX.len()..], params)
}

// 普通文本
fn plain_text(text: &str) -> RichText {
  RichText {
    segments: vec![RichTextSegment {
      text: text.to_string(),
      style: TextStyle::default(),
    }],
  }
}

// 格式化富文本内容
fn parse_formatted_text(text: &str, params: Option<&RichTextParams>) -> RichText {
  // 片段
  let mut segments = Vec::new();
  // 输出
  let mut output = String::new();
  // 当前样式（进入先重置一下）
  let mut current_style = TextStyle::default();
  // 可迭代字符对象
  let mut chars = text.chars().peekable();

  while let Some(ch) = chars.next() {
    // 转义字符
    if ch == '\\' {
      if let Some(escaped) = read_escaped_char(&mut chars) {
        output.push(escaped);
      } else {
        output.push(ch);
      }
      continue;
    }

    // 动态参数
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

    // 样式标签解析
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

// 读取参数名称
fn read_parameter_name<I>(chars: &mut std::iter::Peekable<I>) -> ParameterReadResult
where
  I: Iterator<Item = char>,
{
  let mut name = String::new();

  while let Some(next) = chars.peek() {
    // 新的{表示当前参数未闭合，被破坏，不再继续向下读取，而是跳出后等待新的消费
    if *next == '{' {
      return ParameterReadResult::Broken(name);
    }

    // 到这里才真正消费字符
    let ch = chars.next().unwrap();

    // 参数内部转义
    if ch == '\\' {
      if let Some(escaped) = read_escaped_char(chars) {
        name.push(escaped);
      } else {
        name.push(ch);
      }
      continue;
    }

    // 正常闭合
    if ch == '}' {
      return ParameterReadResult::Closed(name);
    }

    name.push(ch);
  }

  ParameterReadResult::Broken(name)
}

// 写入参数替换结果
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

// 替换键
fn resolve_parameter(name: &str, params: Option<&RichTextParams>) -> Option<String> {
  params.and_then(|params| params.get(name)).cloned()
}

// 解析标签
fn read_tag<I>(chars: &mut std::iter::Peekable<I>) -> TagReadResult
where
  I: Iterator<Item = char>,
{
  // 创建标签项
  let mut tag = String::new();

  // 开始循环检查内部标签项
  while let Some(next) = chars.peek() {
    // 若遇到未转义的<，说明标签未闭合，解析新的标签
    if *next == '<' {
      return TagReadResult::Broken(tag);
    }

    // 下一个并处理小写，避免大小写判定失效
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

// 字符转义
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

// 刷新缓冲区为富文本片段
fn flush_segment(segments: &mut Vec<RichTextSegment>, output: &mut String, style: &TextStyle) {
  if output.is_empty() {
    return;
  }
  segments.push(RichTextSegment {
    text: std::mem::take(output),
    style: style.clone(),
  });
}

// 标签解析
fn apply_tag(tag: &str, current_style: &mut TextStyle) -> bool {
  // 标签去空格
  let tag = tag.trim();

  // 重置样式（优先级最高）
  if tag == "reset" {
    current_style.reset();
    return true;
  }

  // 关闭前景色
  if tag == "/fg" {
    current_style.clear_foreground();
    return true;
  }

  // 关闭背景色
  if tag == "/bg" {
    current_style.clear_background();
    return true;
  }

  // 解析/开头的样式关闭指令
  if let Some(style_name) = tag.strip_prefix('/') {
    return current_style.disable_style(style_name.trim());
  }

  // 设置前景色
  if let Some(color_value) = tag.strip_prefix("fg:") {
    if let Some(color) = parse_text_color(color_value) {
      current_style.set_foreground(color);
      return true;
    }
    return false;
  }

  // 设置背景色
  if let Some(color_value) = tag.strip_prefix("bg:") {
    if let Some(color) = parse_text_color(color_value) {
      current_style.set_background(color);
      return true;
    }
    return false;
  }

  // 输出启用的样式
  current_style.enable_style(tag)
}
