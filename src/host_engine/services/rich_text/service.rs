use super::{parser, RichText, RichTextParams};

pub struct RichTextService;

impl RichTextService {
  pub fn new() -> Self {
    Self
  }

  pub fn parse(&self, text: &str, params: Option<&RichTextParams>) -> RichText {
    parser::parse(text, params)
  }

  /// 提取纯可见文本（去除 f% 前缀、所有富文本标签，并进行模板替换）。
  /// 用于长度测量等场景。
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
