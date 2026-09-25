mod pool;
mod runtime;
mod types;

use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{Arc, RwLock, Weak},
  time::Duration,
};

use crossbeam_channel::Sender;

use crate::host_engine::services::EngineEvent;

pub use pool::AudioObjectPool;
pub use tg_core_audio::{
  AudioAsyncEvent, AudioCaptureId, AudioError, AudioErrorCode, AudioId, AudioPoolId, AudioSource,
  AudioState, AudioTypeId, ResolvedAudioFile,
};
pub use types::{AudioObject, AudioType};

use pool::AudioPoolState;
pub(crate) use runtime::AudioCommand;
use runtime::AudioRuntime;
use types::AudioPlaybackSnapshot;

pub struct AudioService {
  runtime: AudioRuntime,
  pools: HashMap<AudioPoolId, Weak<RwLock<AudioPoolState>>>,
  master_volume: f32,
  globally_paused: bool,
  backend_available: bool,
  active_capture: Option<AudioCaptureId>,
  next_capture_id: u64,
  closed: bool,
}

impl AudioService {
  pub fn new(event_tx: Sender<EngineEvent>) -> Self {
    Self {
      runtime: AudioRuntime::new(event_tx),
      pools: HashMap::new(),
      master_volume: 1.0,
      globally_paused: false,
      backend_available: true,
      active_capture: None,
      next_capture_id: 1,
      closed: false,
    }
  }

  pub fn create_type(
    &mut self,
    pool: &mut AudioObjectPool,
    name: impl Into<String>,
  ) -> Result<AudioTypeId, AudioError> {
    self.ensure_pool(pool)?;
    let mut state = write_pool(&pool.state);
    let placeholder = AudioType {
      id: AudioTypeId {
        pool_id: pool.id(),
        index: 0,
        generation: 0,
      },
      name: name.into(),
      volume: 1.0,
      paused: false,
    };
    let (index, generation) = state.types.insert(placeholder);
    let id = AudioTypeId {
      pool_id: pool.id(),
      index,
      generation,
    };
    if let Some(audio_type) = state.types.get_mut(index, generation) {
      audio_type.id = id;
    }
    Ok(id)
  }

  pub fn remove_type(
    &mut self,
    pool: &mut AudioObjectPool,
    type_id: AudioTypeId,
  ) -> Result<bool, AudioError> {
    self.ensure_matching_pool(pool, type_id.pool_id)?;
    let mut state = write_pool(&pool.state);
    if state.audio_ids(pool.id()).into_iter().any(|id| {
      state
        .objects
        .get(id.index, id.generation)
        .is_some_and(|audio| audio.type_id == Some(type_id))
    }) {
      return Err(AudioError::sanitized(AudioErrorCode::TypeInUse));
    }
    Ok(
      state
        .types
        .remove(type_id.index, type_id.generation)
        .is_some(),
    )
  }

  pub fn set_type_volume(
    &mut self,
    pool: &mut AudioObjectPool,
    type_id: AudioTypeId,
    volume: f32,
  ) -> Result<bool, AudioError> {
    let volume = valid_volume(volume)?;
    self.ensure_matching_pool(pool, type_id.pool_id)?;
    let mut state = write_pool(&pool.state);
    let Some(audio_type) = state.types.get_mut(type_id.index, type_id.generation) else {
      return Ok(false);
    };
    audio_type.volume = volume;
    let ids = matching_audio_ids(&state, pool.id(), type_id);
    for id in ids {
      if let Some(audio) = state.objects.get(id.index, id.generation) {
        self.runtime.send(AudioCommand::SetVolume {
          audio_id: id,
          volume: effective_volume(&state, audio, self.master_volume),
        })?;
      }
    }
    Ok(true)
  }

  pub fn pause_type(
    &mut self,
    pool: &mut AudioObjectPool,
    type_id: AudioTypeId,
  ) -> Result<bool, AudioError> {
    self.ensure_matching_pool(pool, type_id.pool_id)?;
    let mut state = write_pool(&pool.state);
    let Some(audio_type) = state.types.get_mut(type_id.index, type_id.generation) else {
      return Ok(false);
    };
    if audio_type.paused {
      return Ok(true);
    }
    audio_type.paused = true;
    let ids = matching_audio_ids(&state, pool.id(), type_id);
    for id in ids {
      if state
        .objects
        .get(id.index, id.generation)
        .is_some_and(|audio| audio.state == AudioState::Playing)
      {
        self.runtime.send(AudioCommand::Pause { audio_id: id })?;
      }
    }
    Ok(true)
  }

  pub fn resume_type(
    &mut self,
    pool: &mut AudioObjectPool,
    type_id: AudioTypeId,
  ) -> Result<bool, AudioError> {
    self.ensure_matching_pool(pool, type_id.pool_id)?;
    let mut state = write_pool(&pool.state);
    let Some(audio_type) = state.types.get_mut(type_id.index, type_id.generation) else {
      return Ok(false);
    };
    if !audio_type.paused {
      return Ok(true);
    }
    audio_type.paused = false;
    let ids = matching_audio_ids(&state, pool.id(), type_id);
    for id in ids {
      if state
        .objects
        .get(id.index, id.generation)
        .is_some_and(|audio| {
          audio.state == AudioState::Paused && !audio.object_paused && !self.globally_paused
        })
      {
        self.runtime.send(AudioCommand::Resume { audio_id: id })?;
      }
    }
    Ok(true)
  }

  pub fn stop_type(
    &mut self,
    pool: &mut AudioObjectPool,
    type_id: AudioTypeId,
  ) -> Result<bool, AudioError> {
    self.ensure_matching_pool(pool, type_id.pool_id)?;
    let mut state = write_pool(&pool.state);
    if state.types.get(type_id.index, type_id.generation).is_none() {
      return Ok(false);
    }
    for id in matching_audio_ids(&state, pool.id(), type_id) {
      if let Some(audio) = state.objects.get_mut(id.index, id.generation) {
        audio.pending_play = false;
      }
      self.runtime.send(AudioCommand::Stop { audio_id: id })?;
    }
    Ok(true)
  }

  pub fn create(
    &mut self,
    pool: &mut AudioObjectPool,
    source: AudioSource,
    type_id: Option<AudioTypeId>,
  ) -> Result<AudioId, AudioError> {
    self.ensure_pool(pool)?;
    let mut state = write_pool(&pool.state);
    if let Some(type_id) = type_id
      && (type_id.pool_id != pool.id()
        || state.types.get(type_id.index, type_id.generation).is_none())
    {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    }
    let snapshot = Arc::new(AudioPlaybackSnapshot::new());
    let placeholder = AudioObject {
      id: AudioId {
        pool_id: pool.id(),
        index: 0,
        generation: 0,
      },
      source: source.clone(),
      type_id,
      state: AudioState::Loading,
      volume: 1.0,
      looped: false,
      duration: None,
      position: Duration::ZERO,
      pending_play: false,
      object_paused: false,
      snapshot: snapshot.clone(),
    };
    let (index, generation) = state.objects.insert(placeholder);
    let id = AudioId {
      pool_id: pool.id(),
      index,
      generation,
    };
    {
      let audio = state
        .objects
        .get_mut(index, generation)
        .expect("new audio object disappeared");
      audio.id = id;
    }
    let volume = state
      .objects
      .get(index, generation)
      .map(|audio| effective_volume(&state, audio, self.master_volume))
      .unwrap_or(self.master_volume);
    if let Err(error) = self.runtime.send(AudioCommand::Load {
      pool_id: pool.id(),
      audio_id: id,
      source,
      volume,
      looped: false,
      snapshot,
    }) {
      state.objects.remove(index, generation);
      return Err(error);
    }
    Ok(id)
  }

  pub fn remove(
    &mut self,
    pool: &mut AudioObjectPool,
    audio_id: AudioId,
  ) -> Result<bool, AudioError> {
    self.ensure_matching_pool(pool, audio_id.pool_id)?;
    let removed = write_pool(&pool.state)
      .objects
      .remove(audio_id.index, audio_id.generation)
      .is_some();
    if removed {
      self.runtime.send(AudioCommand::Remove { audio_id })?;
    }
    Ok(removed)
  }

  pub(crate) fn remove_owned(&mut self, audio_id: AudioId) -> Result<bool, AudioError> {
    let Some(pool) = self.pools.get(&audio_id.pool_id).and_then(Weak::upgrade) else {
      self.pools.remove(&audio_id.pool_id);
      return Ok(false);
    };
    let removed = write_pool(&pool)
      .objects
      .remove(audio_id.index, audio_id.generation)
      .is_some();
    if removed {
      self.runtime.send(AudioCommand::Remove { audio_id })?;
    }
    Ok(removed)
  }

  pub fn play(&mut self, pool: &mut AudioObjectPool, audio_id: AudioId) -> Result<(), AudioError> {
    self.ensure_matching_pool(pool, audio_id.pool_id)?;
    let mut state = write_pool(&pool.state);
    let Some(current) = state.objects.get(audio_id.index, audio_id.generation) else {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    };
    let type_is_paused = type_paused(&state, current);
    let Some(audio) = state.objects.get_mut(audio_id.index, audio_id.generation) else {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    };
    match audio.state {
      AudioState::Playing => return Ok(()),
      AudioState::Failed => return Err(AudioError::sanitized(AudioErrorCode::InvalidState)),
      AudioState::Paused => {
        audio.object_paused = false;
        let paused = type_is_paused || self.globally_paused;
        if !paused {
          self.runtime.send(AudioCommand::Resume { audio_id })?;
        }
        return Ok(());
      }
      AudioState::Created
      | AudioState::Loading
      | AudioState::Ready
      | AudioState::Stopped
      | AudioState::Finished => {}
    }
    audio.pending_play = true;
    let paused = audio.object_paused || type_is_paused || self.globally_paused;
    self.runtime.send(AudioCommand::Play { audio_id, paused })
  }

  pub fn pause(&mut self, pool: &mut AudioObjectPool, audio_id: AudioId) -> Result<(), AudioError> {
    self.ensure_matching_pool(pool, audio_id.pool_id)?;
    let mut state = write_pool(&pool.state);
    let Some(audio) = state.objects.get_mut(audio_id.index, audio_id.generation) else {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    };
    if audio.object_paused {
      return Ok(());
    }
    audio.object_paused = true;
    if audio.state == AudioState::Playing || audio.pending_play {
      self.runtime.send(AudioCommand::Pause { audio_id })?;
    }
    Ok(())
  }

  pub fn resume(
    &mut self,
    pool: &mut AudioObjectPool,
    audio_id: AudioId,
  ) -> Result<(), AudioError> {
    self.ensure_matching_pool(pool, audio_id.pool_id)?;
    let mut state = write_pool(&pool.state);
    let Some(current) = state.objects.get(audio_id.index, audio_id.generation) else {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    };
    let type_is_paused = type_paused(&state, current);
    let Some(audio) = state.objects.get_mut(audio_id.index, audio_id.generation) else {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    };
    audio.object_paused = false;
    if audio.state == AudioState::Paused && !type_is_paused && !self.globally_paused {
      self.runtime.send(AudioCommand::Resume { audio_id })?;
    }
    Ok(())
  }

  pub fn stop(&mut self, pool: &mut AudioObjectPool, audio_id: AudioId) -> Result<(), AudioError> {
    self.ensure_matching_pool(pool, audio_id.pool_id)?;
    let mut state = write_pool(&pool.state);
    let Some(audio) = state.objects.get_mut(audio_id.index, audio_id.generation) else {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    };
    audio.pending_play = false;
    self.runtime.send(AudioCommand::Stop { audio_id })
  }

  pub fn restart(
    &mut self,
    pool: &mut AudioObjectPool,
    audio_id: AudioId,
  ) -> Result<(), AudioError> {
    self.ensure_matching_pool(pool, audio_id.pool_id)?;
    let mut state = write_pool(&pool.state);
    let Some(current) = state.objects.get(audio_id.index, audio_id.generation) else {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    };
    let type_is_paused = type_paused(&state, current);
    let Some(audio) = state.objects.get_mut(audio_id.index, audio_id.generation) else {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    };
    if audio.state == AudioState::Failed {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidState));
    }
    audio.pending_play = true;
    let paused = audio.object_paused || type_is_paused || self.globally_paused;
    self
      .runtime
      .send(AudioCommand::Restart { audio_id, paused })
  }

  pub fn set_volume(
    &mut self,
    pool: &mut AudioObjectPool,
    audio_id: AudioId,
    volume: f32,
  ) -> Result<bool, AudioError> {
    let volume = valid_volume(volume)?;
    self.ensure_matching_pool(pool, audio_id.pool_id)?;
    let mut state = write_pool(&pool.state);
    let Some(current) = state.objects.get(audio_id.index, audio_id.generation) else {
      return Ok(false);
    };
    let type_volume = current
      .type_id
      .and_then(|type_id| state.types.get(type_id.index, type_id.generation))
      .map(|audio_type| audio_type.volume)
      .unwrap_or(1.0);
    let Some(audio) = state.objects.get_mut(audio_id.index, audio_id.generation) else {
      return Ok(false);
    };
    audio.volume = volume;
    let effective = audio.volume * type_volume * self.master_volume;
    self.runtime.send(AudioCommand::SetVolume {
      audio_id,
      volume: effective,
    })?;
    Ok(true)
  }

  pub fn set_loop(
    &mut self,
    pool: &mut AudioObjectPool,
    audio_id: AudioId,
    looped: bool,
  ) -> Result<bool, AudioError> {
    self.ensure_matching_pool(pool, audio_id.pool_id)?;
    let mut state = write_pool(&pool.state);
    let Some(audio) = state.objects.get_mut(audio_id.index, audio_id.generation) else {
      return Ok(false);
    };
    audio.looped = looped;
    self
      .runtime
      .send(AudioCommand::SetLoop { audio_id, looped })?;
    Ok(true)
  }

  pub fn pause_all(&mut self) -> Result<bool, AudioError> {
    if self.globally_paused {
      return Ok(true);
    }
    self.globally_paused = true;
    let mut affected = false;
    for pool in self.live_pools() {
      let state = read_pool(&pool);
      for (index, generation) in state.objects.keys() {
        if let Some(audio) = state.objects.get(index, generation)
          && audio.state == AudioState::Playing
        {
          self
            .runtime
            .send(AudioCommand::Pause { audio_id: audio.id })?;
          affected = true;
        }
      }
    }
    Ok(affected)
  }

  pub fn resume_all(&mut self) -> Result<bool, AudioError> {
    if !self.globally_paused {
      return Ok(true);
    }
    self.globally_paused = false;
    let mut affected = false;
    for pool in self.live_pools() {
      let state = read_pool(&pool);
      for (index, generation) in state.objects.keys() {
        let Some(audio) = state.objects.get(index, generation) else {
          continue;
        };
        if audio.state == AudioState::Paused && !effective_paused(&state, audio, false) {
          self
            .runtime
            .send(AudioCommand::Resume { audio_id: audio.id })?;
          affected = true;
        }
      }
    }
    Ok(affected)
  }

  pub fn stop_all(&mut self) -> Result<bool, AudioError> {
    let mut any = false;
    for pool in self.live_pools() {
      let mut state = write_pool(&pool);
      for (index, generation) in state.objects.keys() {
        if let Some(audio) = state.objects.get_mut(index, generation) {
          audio.pending_play = false;
          any = true;
        }
      }
    }
    self.runtime.send(AudioCommand::StopAll)?;
    Ok(any)
  }

  pub fn set_all_volume(&mut self, volume: f32) -> Result<bool, AudioError> {
    self.master_volume = valid_volume(volume)?;
    let mut any = false;
    for pool in self.live_pools() {
      let state = read_pool(&pool);
      for (index, generation) in state.objects.keys() {
        if let Some(audio) = state.objects.get(index, generation) {
          self.runtime.send(AudioCommand::SetVolume {
            audio_id: audio.id,
            volume: effective_volume(&state, audio, self.master_volume),
          })?;
          any = true;
        }
      }
    }
    Ok(any)
  }

  pub fn clear_cache(&self) -> Result<(), AudioError> {
    self.runtime.send(AudioCommand::ClearCache)
  }

  pub fn master_volume(&self) -> f32 {
    self.master_volume
  }

  pub fn backend_available(&self) -> bool {
    self.backend_available
  }

  pub fn state(&self, pool: &AudioObjectPool, audio_id: AudioId) -> Option<AudioState> {
    matching_audio(pool, audio_id).map(|audio| audio.state)
  }

  pub fn is_playing(&self, pool: &AudioObjectPool, audio_id: AudioId) -> bool {
    self.state(pool, audio_id) == Some(AudioState::Playing)
  }

  pub fn is_paused(&self, pool: &AudioObjectPool, audio_id: AudioId) -> bool {
    self.state(pool, audio_id) == Some(AudioState::Paused)
  }

  pub fn is_stopped(&self, pool: &AudioObjectPool, audio_id: AudioId) -> bool {
    self.state(pool, audio_id) == Some(AudioState::Stopped)
  }

  pub fn is_finished(&self, pool: &AudioObjectPool, audio_id: AudioId) -> bool {
    self.state(pool, audio_id) == Some(AudioState::Finished)
  }

  pub fn duration(&self, pool: &AudioObjectPool, audio_id: AudioId) -> Option<Duration> {
    matching_audio(pool, audio_id).and_then(|audio| audio.duration)
  }

  pub fn position(&self, pool: &AudioObjectPool, audio_id: AudioId) -> Option<Duration> {
    matching_audio(pool, audio_id).map(|audio| audio.latest_position())
  }

  pub fn seek(
    &mut self,
    pool: &mut AudioObjectPool,
    audio_id: AudioId,
    position: Duration,
  ) -> Result<(), AudioError> {
    self.ensure_matching_pool(pool, audio_id.pool_id)?;
    if matching_audio(pool, audio_id).is_none() {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    }
    self.runtime.send(AudioCommand::Seek { audio_id, position })
  }

  pub fn start_capture(&mut self, path: PathBuf) -> Result<AudioCaptureId, AudioError> {
    if self.closed {
      return Err(AudioError::sanitized(AudioErrorCode::RuntimeClosed));
    }
    if self.active_capture.is_some() {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidState));
    }
    let capture_id = AudioCaptureId(self.next_capture_id);
    self.next_capture_id = self.next_capture_id.saturating_add(1);
    self
      .runtime
      .send(AudioCommand::StartCapture { capture_id, path })?;
    self.active_capture = Some(capture_id);
    Ok(capture_id)
  }

  pub fn pause_capture(&self, capture_id: AudioCaptureId) -> Result<(), AudioError> {
    if self.active_capture != Some(capture_id) {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    }
    self.runtime.send(AudioCommand::PauseCapture { capture_id })
  }

  pub fn resume_capture(&self, capture_id: AudioCaptureId) -> Result<(), AudioError> {
    if self.active_capture != Some(capture_id) {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    }
    self
      .runtime
      .send(AudioCommand::ResumeCapture { capture_id })
  }

  pub fn stop_capture(&mut self, capture_id: AudioCaptureId) -> Result<(), AudioError> {
    if self.active_capture != Some(capture_id) {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    }
    self
      .runtime
      .send(AudioCommand::StopCapture { capture_id })?;
    self.active_capture = None;
    Ok(())
  }

  pub fn handle_engine_event(&mut self, event: &AudioAsyncEvent) {
    if !event.has_valid_identity() {
      return;
    }
    if matches!(event, AudioAsyncEvent::BackendFailed { .. }) {
      self.backend_available = false;
      return;
    }
    if matches!(
      event,
      AudioAsyncEvent::CaptureSaved { .. } | AudioAsyncEvent::CaptureFailed { .. }
    ) {
      let capture_id = match event {
        AudioAsyncEvent::CaptureSaved { capture_id, .. }
        | AudioAsyncEvent::CaptureFailed { capture_id, .. } => *capture_id,
        _ => unreachable!(),
      };
      if self.active_capture == Some(capture_id) {
        if matches!(event, AudioAsyncEvent::CaptureFailed { .. }) {
          let _ = self.runtime.send(AudioCommand::StopCapture { capture_id });
        }
        self.active_capture = None;
      }
      return;
    }
    let Some(audio_id) = event.audio_id() else {
      return;
    };
    let Some(pool) = self.pools.get(&audio_id.pool_id).and_then(Weak::upgrade) else {
      self.pools.remove(&audio_id.pool_id);
      return;
    };
    let mut state = write_pool(&pool);
    let Some(audio) = state.objects.get_mut(audio_id.index, audio_id.generation) else {
      return;
    };
    match event {
      AudioAsyncEvent::Ready { duration, .. } => {
        audio.duration = Some(*duration);
        if matches!(audio.state, AudioState::Created | AudioState::Loading) {
          audio.state = AudioState::Ready;
        }
      }
      AudioAsyncEvent::Started { position, .. } => {
        audio.state = AudioState::Playing;
        audio.position = *position;
        audio.snapshot.set_position(*position);
        audio.pending_play = false;
      }
      AudioAsyncEvent::Paused { position, .. } => {
        audio.state = AudioState::Paused;
        audio.position = *position;
        audio.snapshot.set_position(*position);
      }
      AudioAsyncEvent::Resumed { position, .. } => {
        audio.state = AudioState::Playing;
        audio.position = *position;
        audio.snapshot.set_position(*position);
      }
      AudioAsyncEvent::Stopped { .. } => {
        audio.state = AudioState::Stopped;
        audio.position = Duration::ZERO;
        audio.snapshot.set_position(Duration::ZERO);
        audio.pending_play = false;
      }
      AudioAsyncEvent::Finished { duration, .. } => {
        audio.state = AudioState::Finished;
        audio.duration = Some(*duration);
        audio.position = *duration;
        audio.snapshot.set_position(*duration);
      }
      AudioAsyncEvent::Failed { .. } => {
        audio.state = AudioState::Failed;
        audio.pending_play = false;
      }
      AudioAsyncEvent::BackendFailed { .. }
      | AudioAsyncEvent::CaptureSaved { .. }
      | AudioAsyncEvent::CaptureFailed { .. } => {}
    }
  }

  pub fn shutdown(&mut self) {
    if self.closed {
      return;
    }
    self.closed = true;
    self.runtime.shutdown();
    self.pools.clear();
  }

  fn ensure_pool(&mut self, pool: &mut AudioObjectPool) -> Result<(), AudioError> {
    if self.closed {
      return Err(AudioError::sanitized(AudioErrorCode::RuntimeClosed));
    }
    self
      .pools
      .entry(pool.id())
      .or_insert_with(|| Arc::downgrade(&pool.state));
    pool.set_release_sender(self.runtime.command_sender());
    Ok(())
  }

  fn ensure_matching_pool(
    &mut self,
    pool: &mut AudioObjectPool,
    pool_id: AudioPoolId,
  ) -> Result<(), AudioError> {
    if pool.id() != pool_id {
      return Err(AudioError::sanitized(AudioErrorCode::InvalidId));
    }
    self.ensure_pool(pool)
  }

  fn live_pools(&mut self) -> Vec<Arc<RwLock<AudioPoolState>>> {
    let mut live = Vec::new();
    self.pools.retain(|_, pool| {
      if let Some(pool) = pool.upgrade() {
        live.push(pool);
        true
      } else {
        false
      }
    });
    live
  }
}

impl Drop for AudioService {
  fn drop(&mut self) {
    if !std::thread::panicking() {
      self.shutdown();
    }
  }
}

fn read_pool(pool: &RwLock<AudioPoolState>) -> std::sync::RwLockReadGuard<'_, AudioPoolState> {
  pool.read().unwrap_or_else(|poison| poison.into_inner())
}

fn write_pool(pool: &RwLock<AudioPoolState>) -> std::sync::RwLockWriteGuard<'_, AudioPoolState> {
  pool.write().unwrap_or_else(|poison| poison.into_inner())
}

fn valid_volume(volume: f32) -> Result<f32, AudioError> {
  volume
    .is_finite()
    .then(|| volume.clamp(0.0, 1.0))
    .ok_or_else(|| AudioError::sanitized(AudioErrorCode::InvalidVolume))
}

fn matching_audio_ids(
  state: &AudioPoolState,
  pool_id: AudioPoolId,
  type_id: AudioTypeId,
) -> Vec<AudioId> {
  state
    .audio_ids(pool_id)
    .into_iter()
    .filter(|id| {
      state
        .objects
        .get(id.index, id.generation)
        .is_some_and(|audio| audio.type_id == Some(type_id))
    })
    .collect()
}

fn type_paused(state: &AudioPoolState, audio: &AudioObject) -> bool {
  audio.type_id.is_some_and(|type_id| {
    state
      .types
      .get(type_id.index, type_id.generation)
      .is_some_and(|audio_type| audio_type.paused)
  })
}

fn effective_paused(state: &AudioPoolState, audio: &AudioObject, globally_paused: bool) -> bool {
  audio.object_paused || type_paused(state, audio) || globally_paused
}

fn effective_volume(state: &AudioPoolState, audio: &AudioObject, master_volume: f32) -> f32 {
  let type_volume = audio
    .type_id
    .and_then(|type_id| state.types.get(type_id.index, type_id.generation))
    .map(|audio_type| audio_type.volume)
    .unwrap_or(1.0);
  audio.volume * type_volume * master_volume
}

fn matching_audio(pool: &AudioObjectPool, audio_id: AudioId) -> Option<AudioObject> {
  if pool.id() != audio_id.pool_id {
    return None;
  }
  read_pool(&pool.state)
    .objects
    .get(audio_id.index, audio_id.generation)
    .cloned()
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use crossbeam_channel::unbounded;

  use super::*;

  fn service() -> AudioService {
    let (events, _receiver) = unbounded();
    AudioService::new(events)
  }

  fn unresolved_source(name: &str) -> AudioSource {
    AudioSource::File(ResolvedAudioFile::new(PathBuf::from(name)))
  }

  #[test]
  fn pool_ids_are_isolated_and_generations_invalidate_removed_types() {
    let mut service = service();
    let mut first = AudioObjectPool::new(AudioPoolId(1));
    let mut second = AudioObjectPool::new(AudioPoolId(2));

    let old = service.create_type(&mut first, "music").unwrap();
    assert!(service.remove_type(&mut first, old).unwrap());
    let new = service.create_type(&mut first, "music").unwrap();
    assert_eq!(old.index, new.index);
    assert_ne!(old.generation, new.generation);
    assert!(!service.remove_type(&mut first, old).unwrap());
    assert!(matches!(
      service.remove_type(&mut second, new),
      Err(AudioError {
        code: AudioErrorCode::InvalidId,
        ..
      })
    ));
    service.shutdown();
  }

  #[test]
  fn referenced_types_cannot_be_removed_and_volumes_are_finite_and_clamped() {
    let mut service = service();
    let mut pool = AudioObjectPool::new(AudioPoolId(3));
    let audio_type = service.create_type(&mut pool, "effects").unwrap();
    let audio = service
      .create(
        &mut pool,
        unresolved_source("missing-audio.wav"),
        Some(audio_type),
      )
      .unwrap();

    assert!(matches!(
      service.remove_type(&mut pool, audio_type),
      Err(AudioError {
        code: AudioErrorCode::TypeInUse,
        ..
      })
    ));
    assert!(service.set_type_volume(&mut pool, audio_type, 2.0).unwrap());
    assert!(service.set_volume(&mut pool, audio, -1.0).unwrap());
    assert!(service.set_all_volume(4.0).unwrap());
    assert_eq!(service.master_volume(), 1.0);
    {
      let state = read_pool(&pool.state);
      assert_eq!(
        state
          .types
          .get(audio_type.index, audio_type.generation)
          .unwrap()
          .volume,
        1.0
      );
      assert_eq!(
        state
          .objects
          .get(audio.index, audio.generation)
          .unwrap()
          .volume,
        0.0
      );
    }
    assert!(matches!(
      service.set_volume(&mut pool, audio, f32::NAN),
      Err(AudioError {
        code: AudioErrorCode::InvalidVolume,
        ..
      })
    ));
    service.shutdown();
  }

  #[test]
  fn engine_events_update_only_the_matching_live_generation() {
    let mut service = service();
    let mut pool = AudioObjectPool::new(AudioPoolId(4));
    let audio = service
      .create(&mut pool, unresolved_source("missing-audio.wav"), None)
      .unwrap();
    service.handle_engine_event(&AudioAsyncEvent::Ready {
      pool_id: pool.id(),
      audio_id: audio,
      duration: Duration::from_millis(900),
    });
    assert_eq!(service.state(&pool, audio), Some(AudioState::Ready));
    assert_eq!(
      service.duration(&pool, audio),
      Some(Duration::from_millis(900))
    );

    assert!(service.remove(&mut pool, audio).unwrap());
    service.handle_engine_event(&AudioAsyncEvent::Finished {
      pool_id: pool.id(),
      audio_id: audio,
      duration: Duration::from_millis(900),
    });
    assert_eq!(service.state(&pool, audio), None);
    service.shutdown();
  }
}
