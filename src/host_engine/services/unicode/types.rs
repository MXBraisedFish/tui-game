
/// 字素信息：包含文本片段和其终端显示宽度。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphemeInfo {

  pub text: String,

  pub display_width: usize,
}

/// 文本方向枚举。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDirection {
  LTR,
  RTL,
  Neutral,
}

/// 双向文本的一个同向运行段（Bidi Run）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BidiRun {
  pub text: String,
  pub graphemes: Vec<GraphemeInfo>,
  pub direction: TextDirection,
}
