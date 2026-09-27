//! Minimal entry: exports a temporary data directory as a zip archive.

use std::path::PathBuf;

use tg_service_export::{ExportFormat, ExportScope, ExportService};
use tg_service_log::LogService;
use tg_service_storage::StorageService;

fn main() {
  let root = create_temp_dir("tg_export_smoke");
  let data = root.join("data");
  std::fs::create_dir_all(&data).expect("create data directory");
  std::fs::write(data.join("hello.txt"), "hello").expect("write sample file");
  let storage = StorageService::from_root_for_test(root.clone());
  let mut log = LogService::new();

  let archive = ExportService::new()
    .export(
      ExportScope::Data,
      &root,
      "smoke",
      ExportFormat::Zip,
      &storage,
      &mut log,
    )
    .expect("export data directory");
  assert!(archive.is_file());

  std::fs::remove_dir_all(&root).expect("clean up temporary directory");
  println!("export ok: wrote {}", archive.display());
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
