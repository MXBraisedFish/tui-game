//! Minimal entry: exports a temporary data directory as a zip archive.

use tg_service_export::{ExportFormat, ExportScope, ExportService};
use tg_service_log::LogService;
use tg_service_storage::StorageService;

fn main() {
  let root = std::env::temp_dir().join(format!("tg_export_smoke_{}", std::process::id()));
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

  let _ = std::fs::remove_dir_all(&root);
  println!("export ok: wrote {}", archive.display());
}
