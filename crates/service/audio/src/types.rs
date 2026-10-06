//! Identifiers, configuration, states, and events shared by this module.

use std::{
  sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
  },
  time::Duration,
};

use tg_core_audio::{AudioId, AudioSource, AudioState, AudioTypeId};

/// The audio type representation used by this module.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `name` - The name used to identify the object or field.
/// * `volume` - The requested volume multiplier.
/// * `paused` - The paused.
#[derive(Clone, Debug)]
pub struct AudioType {
  /// The identifier of the owned object.
  pub id: AudioTypeId,
  /// The name used to identify the object or field.
  pub name: String,
  /// The requested volume multiplier.
  pub volume: f32,
  /// The paused.
  pub paused: bool,
}

/// A retained copy of audio playback data for later inspection or replay.
#[derive(Debug)]
pub(crate) struct AudioPlaybackSnapshot {
  position_micros: AtomicU64,
}

impl AudioPlaybackSnapshot {
  /// Create an audio playback snapshot with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      position_micros: AtomicU64::new(0),
    }
  }

  /// Return the position for the addressed object.
  pub(crate) fn position(&self) -> Duration {
    Duration::from_micros(self.position_micros.load(Ordering::Relaxed))
  }

  /// Update the position used by this audio playback snapshot.
  pub(crate) fn set_position(&self, position: Duration) {
    self.position_micros.store(
      position.as_micros().min(u64::MAX as u128) as u64,
      Ordering::Relaxed,
    );
  }
}

/// The audio object representation used by this module.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `source` - The audio source carried by this audio object.
/// * `type_id` - The identifier of the type.
/// * `state` - The audio state carried by this audio object.
/// * `volume` - The requested volume multiplier.
/// * `looped` - The looped.
/// * `duration` - The time span for the operation.
/// * `position` - The position represented as a duration.
/// * `pending_play` - The pending play.
/// * `object_paused` - The object paused.
/// * `snapshot` - The snapshot.
#[derive(Clone, Debug)]
pub struct AudioObject {
  /// The identifier of the owned object.
  pub id: AudioId,
  /// The audio source carried by this audio object.
  pub source: AudioSource,
  /// The identifier of the type.
  pub type_id: Option<AudioTypeId>,
  /// The audio state carried by this audio object.
  pub state: AudioState,
  /// The requested volume multiplier.
  pub volume: f32,
  /// The looped.
  pub looped: bool,
  /// The time span for the operation.
  pub duration: Option<Duration>,
  /// The position represented as a duration.
  pub position: Duration,
  /// The pending play.
  pub(crate) pending_play: bool,
  /// The object paused.
  pub(crate) object_paused: bool,
  /// The snapshot.
  pub(crate) snapshot: Arc<AudioPlaybackSnapshot>,
}

impl AudioObject {
  /// Return the latest playback position reported by the audio runtime.
  pub(crate) fn latest_position(&self) -> Duration {
    self.snapshot.position()
  }
}
