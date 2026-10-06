//! Language info support for the i18n service.

/// Display metadata for a selectable language package.
///
/// # Fields
///
/// * `code` - The stable error or language code.
/// * `direction` - The direction.
#[derive(Clone, Debug)]
pub struct LanguageInfo {
  /// The stable error or language code.
  pub code: String,
  /// The direction.
  pub direction: String,
}
