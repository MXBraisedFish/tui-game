//! Validated package identities and stable keys for package-scoped data.
//!
//! # Examples
//!
//! ```rust
//! use tg_core_package_id::{PackageId, PackageSource, PackageType};
//!
//! let id = PackageId::new(PackageSource::Mod, PackageType::Game, "example").expect("valid name");
//! assert_eq!(id.storage_key(), "mod/game/example");
//! assert_eq!(PackageId::from_storage_key(&id.storage_key()).expect("valid key"), id);
//! ```

use serde::{Deserialize, Serialize};

/// The official or mod origin of a package.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageSource {
  /// Content originating from official.
  Official,
  /// Content originating from mod.
  Mod,
}

/// The game or screensaver category of a package.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
  /// A Lua game package.
  Game,
  /// A Lua screensaver package.
  Screensaver,
}

/// A validated package identity independent of its title, version, and directory spelling.
///
/// # Fields
///
/// * `source` - The package source carried by this package id.
/// * `package_type` - Whether the package is a game or screensaver.
/// * `mod_id` - The validated package name within its source and type.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct PackageId {
  /// The package source carried by this package id.
  pub source: PackageSource,
  /// Whether the package is a game or screensaver.
  pub package_type: PackageType,
  /// The validated package name within its source and type.
  pub mod_id: String,
}

impl<'de> Deserialize<'de> for PackageId {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Wire {
      source: PackageSource,
      package_type: PackageType,
      mod_id: String,
    }
    let value = Wire::deserialize(deserializer)?;
    Self::new(value.source, value.package_type, value.mod_id).map_err(serde::de::Error::custom)
  }
}

impl PackageId {
  /// Validate a package name and combine it with the source and package type.
  ///
  /// # Arguments
  ///
  /// * `source` - The source value or package origin.
  /// * `package_type` - Whether the package is a game or screensaver.
  /// * `mod_id` - The validated package name within its source and type.
  ///
  /// # Errors
  ///
  /// Return an error when the package name is empty, exceeds 128 bytes, or contains characters
  /// other than ASCII letters, digits, and underscores.
  pub fn new(
    source: PackageSource,
    package_type: PackageType,
    mod_id: impl Into<String>,
  ) -> Result<Self, String> {
    let mod_id = mod_id.into();
    validate_mod_id(&mod_id)?;
    Ok(Self {
      source,
      package_type,
      mod_id,
    })
  }

  /// Return the canonical source/type/name key used for package-scoped persistence.
  pub fn storage_key(&self) -> String {
    format!(
      "{}/{}/{}",
      self.source.as_str(),
      self.package_type.as_str(),
      self.mod_id
    )
  }

  /// Parse a canonical package storage key into its source, type, and validated name.
  ///
  /// # Errors
  ///
  /// Return an error when the key has invalid source/type components or an invalid package name.
  pub fn from_storage_key(value: &str) -> Result<Self, String> {
    let mut parts = value.split('/');
    let source = match parts.next() {
      Some("official") => PackageSource::Official,
      Some("mod") => PackageSource::Mod,
      _ => return Err("package key has an invalid source".to_string()),
    };
    let package_type = match parts.next() {
      Some("game") => PackageType::Game,
      Some("screensaver") => PackageType::Screensaver,
      _ => return Err("package key has an invalid type".to_string()),
    };
    let Some(mod_id) = parts.next() else {
      return Err("package key has no mod_id".to_string());
    };
    if parts.next().is_some() {
      return Err("package key has extra path components".to_string());
    }
    Self::new(source, package_type, mod_id)
  }
}

impl std::fmt::Display for PackageId {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str(&self.storage_key())
  }
}

impl PackageSource {
  /// Return the stable string key for this package source.
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Official => "official",
      Self::Mod => "mod",
    }
  }
}

impl PackageType {
  /// Return the stable string key for this package type.
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Game => "game",
      Self::Screensaver => "screensaver",
    }
  }
}

fn validate_mod_id(mod_id: &str) -> Result<(), String> {
  if mod_id.is_empty() {
    return Err("mod_id is empty".to_string());
  }
  if mod_id.len() > 128 {
    return Err("mod_id exceeds 128 bytes".to_string());
  }
  if !mod_id
    .bytes()
    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
  {
    return Err("mod_id may only contain ASCII letters, digits and '_'".to_string());
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn storage_key_joins_source_type_and_mod_id() {
    let id = PackageId::new(PackageSource::Mod, PackageType::Game, "demo_1").unwrap();
    assert_eq!(id.storage_key(), "mod/game/demo_1");
    assert_eq!(id.to_string(), "mod/game/demo_1");
  }

  #[test]
  fn unsafe_mod_ids_are_rejected() {
    for bad in ["", "../escape", "a/b", r"a\b", "old.id", "old-id", "中"] {
      assert!(
        PackageId::new(PackageSource::Official, PackageType::Screensaver, bad).is_err(),
        "{bad:?}"
      );
    }
  }

  #[test]
  fn deserialize_validates_mod_id() {
    let ok: PackageId =
      serde_json::from_str(r#"{"source":"official","package_type":"screensaver","mod_id":"x"}"#)
        .unwrap();
    assert_eq!(ok.storage_key(), "official/screensaver/x");
    assert!(
      serde_json::from_str::<PackageId>(
        r#"{"source":"mod","package_type":"game","mod_id":"../x"}"#
      )
      .is_err()
    );
    for mod_id in ["old.id", "old-id"] {
      let json = format!(r#"{{"source":"mod","package_type":"game","mod_id":"{mod_id}"}}"#);
      assert!(serde_json::from_str::<PackageId>(&json).is_err());
    }
  }

  #[test]
  fn storage_key_parser_requires_canonical_type_and_mod_id() {
    let id = PackageId::from_storage_key("official/screensaver/sky_line").unwrap();
    assert_eq!(id.storage_key(), "official/screensaver/sky_line");
    for invalid in [
      "official/game/old.id",
      "mod/game/old-id",
      "game/demo",
      "mod/unknown/demo",
      "mod/game/demo/extra",
      "mod/game/",
    ] {
      assert!(PackageId::from_storage_key(invalid).is_err(), "{invalid}");
    }
  }
}
