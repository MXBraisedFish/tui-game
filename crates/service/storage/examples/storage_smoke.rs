//! Minimal entry: creates the storage layout in a temporary application root and reads the
//! default display settings back.

use tg_service_log::LogService;
use tg_service_storage::{DisplaySettingsProfile, StorageService};

fn main() {
  let root = std::env::temp_dir().join(format!("tg-storage-smoke-{}", std::process::id()));
  std::fs::create_dir_all(root.join("assets")).expect("temp root");
  std::env::set_current_dir(&root).expect("enter temp root");

  let mut log = LogService::new();
  let storage = StorageService::new(&mut log);
  assert!(storage.data_dir_path().is_dir(), "data directory is created");
  let fps = storage.display_settings_profile().game_list_fps;
  assert_eq!(fps, DisplaySettingsProfile::default().game_list_fps, "fresh root uses defaults");
  println!("storage ok: root={} fps={fps:?}", storage.root_dir().display());

  std::env::set_current_dir(std::env::temp_dir()).expect("leave temp root");
  std::fs::remove_dir_all(&root).expect("clean temp root");
}
