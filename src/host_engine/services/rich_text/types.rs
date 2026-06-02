use super::style::TextStyle;

// 富文本
#[derive(Clone, Debug)]
pub struct RichText {
  pub segments: Vec<RichTextSegment>, // 结构段
}

// 富文本结构
#[derive(Clone, Debug)]
pub struct RichTextSegment {
  pub text: String,     // 内容
  pub style: TextStyle, // 样式
}
