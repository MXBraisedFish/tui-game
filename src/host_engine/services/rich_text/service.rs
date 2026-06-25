use super::{RichText, RichTextParams, parser};

/// 富文本服务：提供解析和纯文本提取功能。
pub struct RichTextService;

impl RichTextService {
  pub fn new() -> Self {
    Self
  }

  /// 解析富文本字符串，返回包含样式信息的分段列表。
  pub fn parse(&self, text: &str, params: Option<&RichTextParams>) -> RichText {
    parser::parse(text, params)
  }

  /// 解析富文本后仅提取可见文本内容（去除所有样式标签）。
  pub fn visible_text(&self, text: &str, params: Option<&RichTextParams>) -> String {
    if params.is_none() && !text.starts_with("f%") {
      return text.to_string();
    }

    let rich_text = self.parse(text, params);
    let mut result = String::new();
    for segment in &rich_text.segments {
      result.push_str(&segment.text);
    }
    result
  }
}
