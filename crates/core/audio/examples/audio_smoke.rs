//! Minimal entry: builds an audio event and validates its identity and error formatting.

use tg_core_audio::{AudioAsyncEvent, AudioError, AudioErrorCode, AudioId, AudioPoolId};

fn main() {
  let audio_id = AudioId { pool_id: AudioPoolId(7), index: 1, generation: 2 };
  let failed = AudioAsyncEvent::Failed {
    pool_id: AudioPoolId(7),
    audio_id,
    error: AudioError::sanitized(AudioErrorCode::Decode),
  };
  assert!(failed.has_valid_identity());
  assert_eq!(failed.audio_id(), Some(audio_id));
  println!("audio ok: {failed:?}");
}
