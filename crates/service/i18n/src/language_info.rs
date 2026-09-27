/// Information about the current language, derived from `language_registry.json`.
#[derive(Clone, Debug)]
pub struct LanguageInfo {
  pub code: String,
  pub direction: String,
}
