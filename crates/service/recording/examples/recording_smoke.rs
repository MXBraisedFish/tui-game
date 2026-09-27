//! Minimal entry: checks the idle recorder state and that a missing recording file is rejected.

use tg_service_recording::{RecordingService, RecordingState, load_recording_playback};

fn main() {
  let mut recording = RecordingService::new();
  assert_eq!(recording.state(), RecordingState::Stopped);
  assert!(!recording.pause(), "nothing to pause while stopped");
  assert!(!recording.is_recording());

  let missing = std::env::temp_dir().join("tg_recording_smoke_missing.tgr");
  assert!(load_recording_playback(&missing).is_none());
  println!("recording ok: idle recorder, missing file rejected");
}
