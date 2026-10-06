//! Independent storage smoke entry exercising the public API and checking its results.

use std::path::PathBuf;

use tg_core_package_id::{PackageId, PackageSource, PackageType};
use tg_service_log::LogService;
use tg_service_storage::{BestGameSave, DisplaySettingsProfile, StorageService};

fn main() {
  let root = create_temp_dir("tg-storage-smoke");
  std::fs::create_dir_all(root.join("assets")).expect("temp root");

  let mut log = LogService::new();
  let storage = StorageService::new(root.clone(), &mut log).expect("initialize explicit root");
  assert!(
    storage.data_dir_path().is_dir(),
    "data directory is created"
  );
  let fps = storage.display_settings_profile().game_list_fps;
  assert_eq!(
    fps,
    DisplaySettingsProfile::default().game_list_fps,
    "fresh root uses defaults"
  );
  println!(
    "storage ok: root={} fps={fps:?}",
    storage.root_dir().display()
  );

  let id = PackageId::new(PackageSource::Mod, PackageType::Game, "best_smoke").expect("package ID");
  let data = serde_json::json!({
    "best_string":{"type":"i18n","key":"score","callback":"f%Best: {value:score}"},
    "value":{"score":"42"}, "score":42
  });
  let best = BestGameSave::try_from(data.clone()).expect("valid localized best score");
  storage
    .write_best_game_save(&id, best.clone(), &mut log)
    .expect("persist best score");
  storage.reload_game_save_profile(&mut log);
  assert_eq!(storage.best_game_save(&id), Some(best));
  assert_eq!(storage.best_game_save(&id).unwrap().data, data);
  assert!(
    BestGameSave::try_from(serde_json::json!({"best_string":"x","value":{"score":42}})).is_err()
  );
  std::fs::remove_dir_all(&root).expect("clean temp root");
}

fn create_temp_dir(prefix: &str) -> PathBuf {
  let nonce = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default()
    .as_nanos();
  let root = std::env::temp_dir().join(format!("{prefix}_{}_{nonce}", std::process::id()));
  std::fs::create_dir(&root)
    .unwrap_or_else(|error| panic!("create temporary directory {}: {error}", root.display()));
  root
}
