use serde::Deserialize;

// 语言注册表
#[derive(Clone, Debug, Deserialize)]
pub struct LanguageRegistryEntry {
  pub code: String, // 语言代码
  pub name: String, // 语言名称
  pub title: String,  // 语言标题
  pub hint: String // 语言操作提示
}