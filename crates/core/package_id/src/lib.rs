use serde::{Deserialize, Serialize};

/// 包来源（官方或模组）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageSource {
  Official,
  Mod,
}

/// 包类型（游戏或屏保）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
  Game,
  Screensaver,
}

/// 宿主内部使用的稳定包身份。版本、标题和目录名不参与身份计算。
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct PackageId {
  pub source: PackageSource,
  pub package_type: PackageType,
  pub mod_id: String,
}

impl<'de> Deserialize<'de> for PackageId {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    #[derive(Deserialize)]
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

  pub fn storage_key(&self) -> String {
    format!(
      "{}/{}/{}",
      self.source.as_str(),
      self.package_type.as_str(),
      self.mod_id
    )
  }
}

impl std::fmt::Display for PackageId {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str(&self.storage_key())
  }
}

impl PackageSource {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Official => "official",
      Self::Mod => "mod",
    }
  }
}

impl PackageType {
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
    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
  {
    return Err("mod_id may only contain ASCII letters, digits, '.', '_' and '-'".to_string());
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
    for bad in ["", "../escape", "a/b", r"a\b"] {
      assert!(PackageId::new(PackageSource::Official, PackageType::Screensaver, bad).is_err(), "{bad:?}");
    }
  }

  #[test]
  fn deserialize_validates_mod_id() {
    let ok: PackageId =
      serde_json::from_str(r#"{"source":"official","package_type":"screensaver","mod_id":"x"}"#).unwrap();
    assert_eq!(ok.storage_key(), "official/screensaver/x");
    assert!(
      serde_json::from_str::<PackageId>(r#"{"source":"mod","package_type":"game","mod_id":"../x"}"#)
        .is_err()
    );
  }
}
