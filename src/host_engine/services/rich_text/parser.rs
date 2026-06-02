use super::RichTextParams;
use super::{RichText, RichTextSegment, TextStyle};

// 富文本前缀
const RICH_TEXT_PREFIX: &str = "f%";

// 解析入口
pub fn parse(text: &str, params: Option<&RichTextParams>) -> RichText {
  // 前缀判断
  if !text.starts_with(RICH_TEXT_PREFIX) {
    // 如果没有富文本前缀，返回普通文本
    return plain_text(text);
  }

  // 进行富文本解析
  parse_formatted_text(&text[RICH_TEXT_PREFIX.len()..])
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
fn parse_formatted_text(text: &str) -> RichText {
  let mut output = String::new();
  let mut chars = text.chars().peekable();

  while let Some(ch) = chars.next() {
    if ch == '\\' {
      match chars.peek() {
        Some('{') | Some('}') | Some('<') | Some('>') | Some('\\') => {
          if let Some(escaped) = chars.next() {
            output.push(escaped);
          }
        }
        _ => {
          output.push(ch);
        }
      }
      continue;
    }

    output.push(ch);
  }

  RichText {
    segments: vec![RichTextSegment {
      text: output,
      style: TextStyle::default(),
    }],
  }
}
