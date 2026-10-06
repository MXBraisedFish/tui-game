//! Generational object storage and ownership-based lookup and removal.

use std::sync::{Arc, RwLock};

use crossbeam_channel::Sender;

use tg_core_arena::Arena;

use super::{AudioCommand, AudioId, AudioObject, AudioPoolId, AudioType};

/// The retained state of audio pool.
///
/// # Fields
///
/// * `types` - The types.
/// * `objects` - The session or page object pool.
pub(crate) struct AudioPoolState {
  /// The types.
  pub(crate) types: Arena<AudioType>,
  /// The session or page object pool.
  pub(crate) objects: Arena<AudioObject>,
}

impl AudioPoolState {
  fn new() -> Self {
    Self {
      types: Arena::new(),
      objects: Arena::new(),
    }
  }

  /// Collect the identities of live playback objects in the audio pool.
  pub(crate) fn audio_ids(&self, pool_id: AudioPoolId) -> Vec<AudioId> {
    self
      .objects
      .keys()
      .into_iter()
      .map(|(index, generation)| AudioId {
        pool_id,
        index,
        generation,
      })
      .collect()
  }
}

/// Owned audio object instances with live-identifier lookup and removal.
///
/// # Fields
///
/// * `state` - The arc carried by this audio object pool.
pub struct AudioObjectPool {
  id: AudioPoolId,
  /// The arc carried by this audio object pool.
  pub(crate) state: Arc<RwLock<AudioPoolState>>,
  release_tx: Option<Sender<AudioCommand>>,
}

impl AudioObjectPool {
  /// Create an audio object pool initialized from `id`.
  pub fn new(id: AudioPoolId) -> Self {
    Self {
      id,
      state: Arc::new(RwLock::new(AudioPoolState::new())),
      release_tx: None,
    }
  }

  /// Return the current id.
  pub fn id(&self) -> AudioPoolId {
    self.id
  }

  /// Install the sender used when an owned audio handle is dropped.
  pub(crate) fn set_release_sender(&mut self, sender: Sender<AudioCommand>) {
    self.release_tx.get_or_insert(sender);
  }
}

impl Drop for AudioObjectPool {
  fn drop(&mut self) {
    if let Some(sender) = &self.release_tx {
      let _ = sender.send(AudioCommand::ReleasePool { pool_id: self.id });
    }
  }
}
