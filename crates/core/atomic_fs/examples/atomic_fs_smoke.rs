//! Minimal entry: atomically writes one file in the temp directory and reads it back.

fn main() {
  let path = std::env::temp_dir().join(format!("tg_atomic_fs_smoke_{}.txt", std::process::id()));
  tg_core_atomic_fs::atomic_write(&path, b"smoke", false).expect("atomic write");
  let text = std::fs::read_to_string(&path).expect("read back");
  std::fs::remove_file(&path).expect("clean up");
  assert_eq!(text, "smoke");
  println!("atomic_fs ok: {}", path.display());
}
