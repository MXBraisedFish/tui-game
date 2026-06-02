use super::{RichText, RichTextParams, parser};

pub struct RichTextService;

impl RichTextService {
  pub fn new() -> Self {
    Self
  }

  // 公共解析接口
  pub fn parse(&self, text: &str, params: Option<&RichTextParams>) -> RichText {
    parser::parse(text, params)
  }
}
