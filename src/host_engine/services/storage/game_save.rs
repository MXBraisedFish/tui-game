use std::{collections::BTreeMap, fs, io};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use tg_core_atomic_fs::atomic_write;

use super::StorageService;
use crate::host_engine::services::{LogService, LogSource, PackageId};

const MAX_GAME_SAVE_PROFILE_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameSaveCapabilities {
  pub package_id: PackageId,
  pub save_enabled: bool,
  pub score_enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ContinueGameSave {
  pub package: PackageId,
  pub data: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BestGameSave {
  pub best_string: String,
  pub data: Value,
}

impl TryFrom<Value> for BestGameSave {
  type Error = &'static str;

  fn try_from(data: Value) -> Result<Self, Self::Error> {
    let best_string = data
      .as_object()
      .and_then(|object| object.get("best_string"))
      .and_then(Value::as_str)
      .ok_or("best save must contain string field 'best_string'")?
      .to_string();
    Ok(Self { best_string, data })
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GameSaveProfile {
  pub continue_slot: Option<ContinueGameSave>,
  pub best: BTreeMap<String, BestGameSave>,
}

impl Default for GameSaveProfile {
  fn default() -> Self {
    Self {
      continue_slot: None,
      best: BTreeMap::new(),
    }
  }
}

impl StorageService {
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

  pub fn continue_game_save(&self) -> Option<ContinueGameSave> {
    self.game_save.borrow().continue_slot.clone()
  }

  pub fn best_game_save(&self, package_id: &PackageId) -> Option<BestGameSave> {
    self
      .game_save
      .borrow()
      .best
      .get(&package_id.storage_key())
      .cloned()
  }

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

  pub fn clear_continue_game_save(&self, log: &mut LogService) -> io::Result<()> {
    let mut profile = self.game_save.borrow().clone();
    if profile.continue_slot.is_none() {
      return Ok(());
    }
    profile.continue_slot = None;
    self.write_game_save_profile(profile, log)
  }

  /// Reconciles save capabilities after a successful package scan.
  ///
  /// A missing or no-longer-saveable continue target cannot be launched and is
  /// removed. Best records are removed only for installed packages that now
  /// explicitly disable score support; temporarily missing packages retain
  /// their records.
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
    let content = serde_json::to_vec_pretty(&profile).map_err(io::Error::other)?;
    if content.len() as u64 > MAX_GAME_SAVE_PROFILE_BYTES {
      return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "game save profile exceeds size limit",
      ));
    }
    atomic_write(&self.profile_game_save_path(), &content, true).map_err(|error| {
      log.error_operation_failed(
        LogSource::Storage,
        "write_profile",
        "game_save",
        error.to_string(),
      );
      error
    })?;
    *self.game_save.borrow_mut() = profile;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{PackageSource, PackageType};

  #[test]
  fn continue_slot_is_shared_and_best_records_are_per_game() {
    let root = std::env::temp_dir().join(format!("tui-game-save-profile-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("data/profiles")).unwrap();
    let storage = StorageService::from_root_for_test(root.clone());
    let mut log = LogService::new();

    storage
      .write_continue_game_save(
        &PackageId::new(PackageSource::Official, PackageType::Game, "game.a").unwrap(),
        serde_json::json!({"level": 1}),
        &mut log,
      )
      .unwrap();
    storage
      .write_continue_game_save(
        &PackageId::new(PackageSource::Mod, PackageType::Game, "game.b").unwrap(),
        serde_json::json!({"level": 2}),
        &mut log,
      )
      .unwrap();
    let slot = storage.continue_game_save().unwrap();
    assert_eq!(slot.package.source, PackageSource::Mod);
    assert_eq!(slot.package.mod_id, "game.b");

    let official = PackageId::new(PackageSource::Official, PackageType::Game, "game.a").unwrap();
    storage
      .write_best_game_save(
        &official,
        BestGameSave {
          best_string: "100".to_string(),
          data: serde_json::json!({"best_string": "100", "score": 100}),
        },
        &mut log,
      )
      .unwrap();
    assert_eq!(
      storage.best_game_save(&official).unwrap().best_string,
      "100"
    );
    let mod_package = PackageId::new(PackageSource::Mod, PackageType::Game, "game.a").unwrap();
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
    let id = PackageId::new(PackageSource::Mod, PackageType::Game, "game.changed").unwrap();

    storage
      .write_continue_game_save(&id, serde_json::json!({"level": 1}), &mut log)
      .unwrap();
    storage
      .write_best_game_save(
        &id,
        BestGameSave {
          best_string: "100".to_string(),
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
