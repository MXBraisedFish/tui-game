//! Independent atomic fs smoke entry exercising the public API and checking its results.

use std::path::PathBuf;

fn main() {
  let root = create_temp_dir("tg_atomic_fs_smoke");
  let path = root.join("smoke.txt");
  tg_core_atomic_fs::atomic_write(&path, b"smoke", false).expect("atomic write");
  let text = std::fs::read_to_string(&path).expect("read back");
  assert_eq!(text, "smoke");
  std::fs::remove_dir_all(&root).expect("clean up temporary directory");
  println!("atomic_fs ok: {}", path.display());
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
