/// 当前语言信息，由 language_registry.json 派生。
#[derive(Clone, Debug)]
pub struct LanguageInfo {
  pub code: String,
  pub direction: String,
}
