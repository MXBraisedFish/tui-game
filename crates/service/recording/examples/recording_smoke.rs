//! Minimal entry: checks the idle recorder state and that a missing recording file is rejected.

use std::path::PathBuf;

use tg_service_recording::{RecordingService, RecordingState, load_recording_playback};

fn main() {
  let mut recording = RecordingService::new();
  assert_eq!(recording.state(), RecordingState::Stopped);
  assert!(!recording.pause(), "nothing to pause while stopped");
  assert!(!recording.is_recording());

  let root = create_temp_dir("tg_recording_smoke");
  let missing = root.join("missing.tgr");
  assert!(load_recording_playback(&missing).is_none());
  std::fs::remove_dir_all(&root).expect("clean up temporary directory");
  println!("recording ok: idle recorder, missing file rejected");
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
