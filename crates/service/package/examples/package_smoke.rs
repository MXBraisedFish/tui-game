//! Minimal entry: scans an empty package root and finds no packages.

use tg_service_log::LogService;
use tg_service_package::PackageService;

fn main() {
  let root = std::env::temp_dir().join(format!("tg_package_smoke_{}", std::process::id()));
  std::fs::create_dir_all(&root).expect("create package root");
  let mut packages = PackageService::new();
  let mut log = LogService::new();

  packages.scan_all(&root, &mut log, "en_us", "missing: {key}");
  assert_eq!(packages.total_count(), 0);
  assert!(packages.game_list().is_empty());

  let _ = std::fs::remove_dir_all(&root);
  println!("package ok: empty root scanned");
}
