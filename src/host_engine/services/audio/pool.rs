use std::sync::{Arc, RwLock};

use crossbeam_channel::Sender;

use tg_core_arena::Arena;

use super::{AudioCommand, AudioId, AudioObject, AudioPoolId, AudioType};

pub(crate) struct AudioPoolState {
  pub(crate) types: Arena<AudioType>,
  pub(crate) objects: Arena<AudioObject>,
}

impl AudioPoolState {
  fn new() -> Self {
    Self {
      types: Arena::new(),
      objects: Arena::new(),
    }
  }

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

pub struct AudioObjectPool {
  id: AudioPoolId,
  pub(crate) state: Arc<RwLock<AudioPoolState>>,
  release_tx: Option<Sender<AudioCommand>>,
}

impl AudioObjectPool {
  pub(crate) fn new(id: AudioPoolId) -> Self {
    Self {
      id,
      state: Arc::new(RwLock::new(AudioPoolState::new())),
      release_tx: None,
    }
  }

  pub fn id(&self) -> AudioPoolId {
    self.id
  }

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
