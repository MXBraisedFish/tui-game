use super::style::TextStyle;

/// 富文本解析结果：由多个带样式的文本段组成。
#[derive(Clone, Debug)]
pub struct RichText {
  pub segments: Vec<RichTextSegment>,
}

/// 富文本的一个样式段：包含文本内容和对应的 TextStyle。
#[derive(Clone, Debug)]
pub struct RichTextSegment {
  pub text: String,
  pub style: TextStyle,
}
