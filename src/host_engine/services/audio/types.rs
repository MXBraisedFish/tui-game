use std::{
  sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
  },
  time::Duration,
};

use tg_core_audio::{AudioId, AudioSource, AudioState, AudioTypeId};

#[derive(Clone, Debug)]
pub struct AudioType {
  pub id: AudioTypeId,
  pub name: String,
  pub volume: f32,
  pub paused: bool,
}

#[derive(Debug)]
pub(crate) struct AudioPlaybackSnapshot {
  position_micros: AtomicU64,
}

impl AudioPlaybackSnapshot {
  pub(crate) fn new() -> Self {
    Self {
      position_micros: AtomicU64::new(0),
    }
  }

  pub(crate) fn position(&self) -> Duration {
    Duration::from_micros(self.position_micros.load(Ordering::Relaxed))
  }

  pub(crate) fn set_position(&self, position: Duration) {
    self.position_micros.store(
      position.as_micros().min(u64::MAX as u128) as u64,
      Ordering::Relaxed,
    );
  }
}

#[derive(Clone, Debug)]
pub struct AudioObject {
  pub id: AudioId,
  pub source: AudioSource,
  pub type_id: Option<AudioTypeId>,
  pub state: AudioState,
  pub volume: f32,
  pub looped: bool,
  pub duration: Option<Duration>,
  pub position: Duration,
  pub(crate) pending_play: bool,
  pub(crate) object_paused: bool,
  pub(crate) snapshot: Arc<AudioPlaybackSnapshot>,
}

impl AudioObject {
  pub(crate) fn latest_position(&self) -> Duration {
    self.snapshot.position()
  }
}
