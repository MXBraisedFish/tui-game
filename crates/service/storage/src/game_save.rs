//! Package-scoped continue saves and best scores with capability-aware persistence.

use std::{collections::BTreeMap, fs, io};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use tg_core_atomic_fs::atomic_write;

use super::StorageService;
use tg_core_package_id::PackageId;
use tg_service_log::{LogService, LogSource};

const MAX_GAME_SAVE_PROFILE_BYTES: u64 = 16 * 1024 * 1024;

/// The game save capabilities representation used by this module.
///
/// # Fields
///
/// * `package_id` - The stable source, type, and name of the package.
/// * `save_enabled` - The save enabled.
/// * `score_enabled` - The score enabled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameSaveCapabilities {
  /// The stable source, type, and name of the package.
  pub package_id: PackageId,
  /// The save enabled.
  pub save_enabled: bool,
  /// The score enabled.
  pub score_enabled: bool,
}

/// The continue game save representation used by this module.
///
/// # Fields
///
/// * `package` - The validated package snapshot.
/// * `data` - The data.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ContinueGameSave {
  /// The validated package snapshot.
  pub package: PackageId,
  /// The data.
  pub data: Value,
}

/// The best game save representation used by this module.
///
/// # Fields
///
/// * `best_string` - The original literal or package translation reference.
/// * `data` - The data.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BestGameSave {
  /// The original literal or package translation reference, retained without localization.
  pub best_string: Value,
  /// The data.
  pub data: Value,
}

impl TryFrom<Value> for BestGameSave {
  type Error = String;

  fn try_from(data: Value) -> Result<Self, Self::Error> {
    tg_service_package::validate_best_save(&data)?;
    let best_string = data["best_string"].clone();
    Ok(Self { best_string, data })
  }
}

/// Persisted user settings for game save.
///
/// # Fields
///
/// * `continue_slot` - The continue slot.
/// * `best` - The best indexed by their declared keys.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GameSaveProfile {
  /// The continue slot.
  pub continue_slot: Option<ContinueGameSave>,
  /// The best indexed by their declared keys.
  #[serde(deserialize_with = "deserialize_best_saves")]
  pub best: BTreeMap<String, BestGameSave>,
}

fn deserialize_best_saves<'de, D>(
  deserializer: D,
) -> Result<BTreeMap<String, BestGameSave>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  let best = BTreeMap::<String, BestGameSave>::deserialize(deserializer)?;
  for saved in best.values() {
    let parsed = BestGameSave::try_from(saved.data.clone()).map_err(serde::de::Error::custom)?;
    if parsed.best_string != saved.best_string {
      return Err(serde::de::Error::custom(
        "best save template does not match its data",
      ));
    }
  }
  for key in best.keys() {
    let package_id = PackageId::from_storage_key(key).map_err(serde::de::Error::custom)?;
    if package_id.package_type != tg_core_package_id::PackageType::Game {
      return Err(serde::de::Error::custom(format!(
        "best save key {key:?} must refer to a game"
      )));
    }
  }
  Ok(best)
}

impl StorageService {
  /// Reload package-scoped save settings and cached save capabilities.
  pub fn reload_game_save_profile(&self, log: &mut LogService) {
    let path = self.profile_game_save_path();
    let profile = fs::metadata(&path)
      .and_then(|metadata| {
        if metadata.len() > MAX_GAME_SAVE_PROFILE_BYTES {
          return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "game save profile exceeds size limit",
          ));
        }
        fs::read_to_string(&path)
      })
      .and_then(|content| serde_json::from_str(&content).map_err(io::Error::other))
      .unwrap_or_else(|error| {
        if error.kind() != io::ErrorKind::NotFound {
          log.warn_operation_failed(
            LogSource::Storage,
            "load_profile",
            "game_save",
            error.to_string(),
          );
        }
        GameSaveProfile::default()
      });
    *self.game_save.borrow_mut() = profile;
  }

  /// Return the retained continue-save value for the package when available.
  pub fn continue_game_save(&self) -> Option<ContinueGameSave> {
    self.game_save.borrow().continue_slot.clone()
  }

  /// Return the retained best score for the package when available.
  pub fn best_game_save(&self, package_id: &PackageId) -> Option<BestGameSave> {
    self
      .game_save
      .borrow()
      .best
      .get(&package_id.storage_key())
      .cloned()
  }

  /// Validate and atomically persist the package's continue-save data.
  ///
  /// # Arguments
  ///
  /// * `package_id` - The stable source, type, and name of the package.
  /// * `data` - The data.
  /// * `log` - The service receiving diagnostic records.
  ///
  /// # Errors
  ///
  /// Return an error for invalid save data or failures while atomically persisting the updated
  /// profile.
  pub fn write_continue_game_save(
    &self,
    package_id: &PackageId,
    data: Value,
    log: &mut LogService,
  ) -> io::Result<()> {
    let mut profile = self.game_save.borrow().clone();
    profile.continue_slot = Some(ContinueGameSave {
      package: package_id.clone(),
      data,
    });
    self.write_game_save_profile(profile, log)
  }

  /// Remove the package's continue-save value and persist the updated profile.
  ///
  /// # Errors
  ///
  /// Return an error when the updated game-save profile cannot be persisted.
  pub fn clear_continue_game_save(&self, log: &mut LogService) -> io::Result<()> {
    let mut profile = self.game_save.borrow().clone();
    if profile.continue_slot.is_none() {
      return Ok(());
    }
    profile.continue_slot = None;
    self.write_game_save_profile(profile, log)
  }

  /// Remove save data that is no longer supported by the package's current declared capabilities.
  ///
  /// # Errors
  ///
  /// Return an error when capability changes require a profile update that cannot be persisted.
  pub fn reconcile_game_save_capabilities(
    &self,
    games: &[GameSaveCapabilities],
    log: &mut LogService,
  ) -> io::Result<()> {
    let mut profile = self.game_save.borrow().clone();
    let original = profile.clone();

    if let Some(slot) = profile.continue_slot.as_ref() {
      let can_continue = games
        .iter()
        .find(|game| game.package_id == slot.package)
        .is_some_and(|game| game.save_enabled);
      if !can_continue {
        profile.continue_slot = None;
      }
    }

    for game in games {
      if !game.score_enabled {
        profile.best.remove(&game.package_id.storage_key());
      }
    }

    if profile == original {
      Ok(())
    } else {
      self.write_game_save_profile(profile, log)
    }
  }

  /// Update the persisted best score according to the package's configured score ordering.
  ///
  /// # Arguments
  ///
  /// * `package_id` - The stable source, type, and name of the package.
  /// * `best` - The best.
  /// * `log` - The service receiving diagnostic records.
  ///
  /// # Errors
  ///
  /// Return an error for invalid score data or failures while persisting the selected best score.
  pub fn write_best_game_save(
    &self,
    package_id: &PackageId,
    best: BestGameSave,
    log: &mut LogService,
  ) -> io::Result<()> {
    let mut profile = self.game_save.borrow().clone();
    profile.best.insert(package_id.storage_key(), best);
    self.write_game_save_profile(profile, log)
  }

  /// Persist the game's supported continue-save and best-score results.
  ///
  /// # Arguments
  ///
  /// * `package_id` - The stable source, type, and name of the package.
  /// * `game` - The game.
  /// * `best` - The best.
  /// * `log` - The service receiving diagnostic records.
  ///
  /// # Errors
  ///
  /// Return an error for invalid game results or failures while persisting the supported save
  /// values.
  pub fn write_game_results(
    &self,
    package_id: &PackageId,
    game: Option<Value>,
    best: Option<BestGameSave>,
    log: &mut LogService,
  ) -> io::Result<()> {
    if game.is_none() && best.is_none() {
      return Ok(());
    }
    let mut profile = self.game_save.borrow().clone();
    if let Some(data) = game {
      profile.continue_slot = Some(ContinueGameSave {
        package: package_id.clone(),
        data,
      });
    }
    if let Some(best) = best {
      profile.best.insert(package_id.storage_key(), best);
    }
    self.write_game_save_profile(profile, log)
  }

  fn write_game_save_profile(
    &self,
    profile: GameSaveProfile,
    log: &mut LogService,
  ) -> io::Result<()> {
    for saved in profile.best.values() {
      let parsed = BestGameSave::try_from(saved.data.clone())
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;
      if parsed.best_string != saved.best_string {
        return Err(io::Error::new(
          io::ErrorKind::InvalidData,
          "best save template does not match its data",
        ));
      }
    }
    let content = serde_json::to_vec_pretty(&profile).map_err(io::Error::other)?;
    if content.len() as u64 > MAX_GAME_SAVE_PROFILE_BYTES {
      return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "game save profile exceeds size limit",
      ));
    }
    atomic_write(&self.profile_game_save_path(), &content, true).inspect_err(|error| {
      log.error_operation_failed(
        LogSource::Storage,
        "write_profile",
        "game_save",
        error.to_string(),
      );
    })?;
    *self.game_save.borrow_mut() = profile;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tg_core_package_id::{PackageSource, PackageType};

  #[test]
  fn localized_best_saves_round_trip_without_freezing_translations() {
    let root = std::env::temp_dir().join(format!("tg-localized-best-{}", std::process::id()));
    fs::create_dir_all(root.join("data/profiles")).unwrap();
    let storage = StorageService::from_root_for_test(root.clone());
    let mut log = LogService::new();
    let id = PackageId::new(PackageSource::Mod, PackageType::Game, "localized").unwrap();
    let data = serde_json::json!({
      "best_string": {"type":"i18n", "key":"score", "callback":"f%Best: {value:score}"},
      "value": {"score":"42", "rank":{"type":"i18n", "key":"rank", "callback":"Gold"}},
      "score":42
    });
    let best = BestGameSave::try_from(data.clone()).unwrap();
    storage
      .write_best_game_save(&id, best.clone(), &mut log)
      .unwrap();
    storage.reload_game_save_profile(&mut log);
    assert_eq!(storage.best_game_save(&id), Some(best));
    assert_eq!(storage.best_game_save(&id).unwrap().data, data);
    assert!(BestGameSave::try_from(serde_json::json!({"best_string":false})).is_err());
    assert!(
      BestGameSave::try_from(serde_json::json!({"best_string":"x", "value":{"score":42}})).is_err()
    );
    assert_eq!(storage.best_game_save(&id).unwrap().data, data);
    let malformed = BestGameSave {
      best_string: "wrong".into(),
      data: serde_json::json!({"best_string":"42"}),
    };
    assert!(
      storage
        .write_best_game_save(&id, malformed, &mut log)
        .is_err()
    );
    storage.reload_game_save_profile(&mut log);
    assert_eq!(storage.best_game_save(&id).unwrap().data, data);
    fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn persisted_best_saves_accept_strings_and_reject_mismatched_templates() {
    let key = PackageId::new(PackageSource::Mod, PackageType::Game, "legacy")
      .unwrap()
      .storage_key();
    let mut json = serde_json::json!({"continue_slot":null,"best":{key.clone():{"best_string":"42","data":{"best_string":"42","score":42}}}});
    assert!(serde_json::from_value::<GameSaveProfile>(json.clone()).is_ok());
    json["best"][&key]["best_string"] = Value::String("different".into());
    assert!(serde_json::from_value::<GameSaveProfile>(json).is_err());
  }

  #[test]
  fn game_save_profile_rejects_legacy_package_ids() {
    for key in ["mod/game/old.id", "official/game/old-id"] {
      let profile = format!(
        r#"{{"continue_slot":null,"best":{{"{key}":{{"best_string":"1","data":{{"best_string":"1"}}}}}}}}"#
      );
      assert!(
        serde_json::from_str::<GameSaveProfile>(&profile).is_err(),
        "{key}"
      );
    }

    let old_continue = r#"{
      "continue_slot":{"package":{"source":"mod","package_type":"game","mod_id":"old.id"},"data":{}},
      "best":{}
    }"#;
    assert!(serde_json::from_str::<GameSaveProfile>(old_continue).is_err());
  }

  #[test]
  fn continue_slot_is_shared_and_best_records_are_per_game() {
    let root = std::env::temp_dir().join(format!("tui-game-save-profile-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("data/profiles")).unwrap();
    let storage = StorageService::from_root_for_test(root.clone());
    let mut log = LogService::new();

    storage
      .write_continue_game_save(
        &PackageId::new(PackageSource::Official, PackageType::Game, "game_a").unwrap(),
        serde_json::json!({"level": 1}),
        &mut log,
      )
      .unwrap();
    storage
      .write_continue_game_save(
        &PackageId::new(PackageSource::Mod, PackageType::Game, "game_b").unwrap(),
        serde_json::json!({"level": 2}),
        &mut log,
      )
      .unwrap();
    let slot = storage.continue_game_save().unwrap();
    assert_eq!(slot.package.source, PackageSource::Mod);
    assert_eq!(slot.package.mod_id, "game_b");

    let official = PackageId::new(PackageSource::Official, PackageType::Game, "game_a").unwrap();
    storage
      .write_best_game_save(
        &official,
        BestGameSave {
          best_string: "100".into(),
          data: serde_json::json!({"best_string": "100", "score": 100}),
        },
        &mut log,
      )
      .unwrap();
    assert_eq!(
      storage.best_game_save(&official).unwrap().best_string,
      "100"
    );
    let mod_package = PackageId::new(PackageSource::Mod, PackageType::Game, "game_a").unwrap();
    assert!(storage.best_game_save(&mod_package).is_none());

    storage.clear_continue_game_save(&mut log).unwrap();
    assert!(storage.continue_game_save().is_none());
    assert_eq!(
      storage.best_game_save(&official).unwrap().best_string,
      "100"
    );

    fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn package_capability_changes_remove_unsupported_save_data() {
    let root = std::env::temp_dir().join(format!(
      "tui-game-save-capability-profile-{}",
      std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("data/profiles")).unwrap();
    let storage = StorageService::from_root_for_test(root.clone());
    let mut log = LogService::new();
    let id = PackageId::new(PackageSource::Mod, PackageType::Game, "game_changed").unwrap();

    storage
      .write_continue_game_save(&id, serde_json::json!({"level": 1}), &mut log)
      .unwrap();
    storage
      .write_best_game_save(
        &id,
        BestGameSave {
          best_string: "100".into(),
          data: serde_json::json!({"best_string": "100"}),
        },
        &mut log,
      )
      .unwrap();

    storage
      .reconcile_game_save_capabilities(
        &[GameSaveCapabilities {
          package_id: id.clone(),
          save_enabled: false,
          score_enabled: false,
        }],
        &mut log,
      )
      .unwrap();

    assert!(storage.continue_game_save().is_none());
    assert!(storage.best_game_save(&id).is_none());
    fs::remove_dir_all(root).unwrap();
  }
}
