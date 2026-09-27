//! Minimal entry: scans an empty package root and finds no packages.

use std::path::PathBuf;

use tg_service_log::LogService;
use tg_service_package::PackageService;

fn main() {
  let root = create_temp_dir("tg_package_smoke");
  let mut packages = PackageService::new();
  let mut log = LogService::new();

  packages.scan_all(&root, &mut log, "en_us", "missing: {key}");
  assert_eq!(packages.total_count(), 0);
  assert!(packages.game_list().is_empty());

  std::fs::remove_dir_all(&root).expect("clean up temporary directory");
  println!("package ok: empty root scanned");
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
