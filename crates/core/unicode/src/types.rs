/// 字素信息：包含文本片段和其终端显示宽度。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphemeInfo {
  pub text: String,

  pub display_width: usize,
}
