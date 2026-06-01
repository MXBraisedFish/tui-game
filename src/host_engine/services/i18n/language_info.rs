use serde::Deserialize;

// 语言信息
#[derive(Clone, Debug, Deserialize)]
pub struct LanguageInfo {
  pub code: String, // 语言代码
  pub direction: String // 语言方向
}