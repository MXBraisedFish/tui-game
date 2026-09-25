use super::{RichText, RichTextParams, parser};

/// 文本解析模式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextMode {
  Auto,
  Plain,
  Rich,
}

impl Default for TextMode {
  fn default() -> Self {
    Self::Auto
  }
}

/// 富文本服务：提供解析和纯文本提取功能。
pub struct RichTextService;

impl RichTextService {
  pub fn new() -> Self {
    Self
  }

  /// 解析富文本字符串，返回包含样式信息的分段列表。
  pub fn parse(&self, text: &str, params: Option<&RichTextParams>) -> RichText {
    parser::parse_auto(text, params)
  }

  pub(crate) fn parse_mode(
    &self,
    text: &str,
    params: Option<&RichTextParams>,
    mode: TextMode,
  ) -> RichText {
    match mode {
      TextMode::Auto => parser::parse_auto(text, params),
      TextMode::Plain => parser::parse_plain(text),
      TextMode::Rich => parser::parse_rich(text, params),
    }
  }

  /// 解析富文本后仅提取可见文本内容（去除所有样式标签）。
  pub fn visible_text(&self, text: &str, params: Option<&RichTextParams>) -> String {
    if params.is_none() && !text.starts_with("f%") {
      return text.to_string();
    }

    // 宿主界面传入参数时已经明确要求格式化；Lua 的 AUTO 模式仍要求 `f%` 前缀。
    let rich_text = if params.is_some() {
      parser::parse_rich(text.strip_prefix("f%").unwrap_or(text), params)
    } else {
      self.parse(text, params)
    };
    let mut result = String::new();
    for segment in &rich_text.segments {
      result.push_str(&segment.text);
    }
    result
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn host_visible_text_resolves_unprefixed_parameters() {
    let mut params = RichTextParams::default();
    params.values.insert("action".into(), "[Enter]".into());

    let rich = RichTextService::new();
    assert_eq!(
      rich.visible_text("{value:action}", Some(&params)),
      "[Enter]"
    );
    assert_eq!(
      rich.visible_text("f%{value:action}", Some(&params)),
      "[Enter]"
    );
  }
}
