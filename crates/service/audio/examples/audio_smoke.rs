//! Independent audio smoke entry exercising the public API and checking its results.

use crossbeam_channel::unbounded;
use tg_core_audio::{AudioAsyncEvent, AudioPoolId};
use tg_service_async::EventSink;
use tg_service_audio::{AudioObjectPool, AudioService};

fn main() {
  let (events, _receiver) = unbounded::<AudioAsyncEvent>();
  let mut audio = AudioService::new(EventSink::new(events));
  let mut pool = AudioObjectPool::new(AudioPoolId(1));

  let music = audio
    .create_type(&mut pool, "music")
    .expect("create audio type");
  audio
    .set_type_volume(&mut pool, music, 0.5)
    .expect("set type volume");
  assert!(
    audio
      .remove_type(&mut pool, music)
      .expect("remove audio type")
  );
  assert!((audio.master_volume() - 1.0).abs() < f32::EPSILON);
  println!("audio ok: type created, adjusted and removed");
}
