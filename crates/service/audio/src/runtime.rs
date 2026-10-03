//! Runtime support for the audio service.

use std::{
  collections::HashMap,
  fs::{self, File},
  io::BufWriter,
  num::{NonZeroU16, NonZeroU32},
  path::PathBuf,
  sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
  },
  thread::{self, JoinHandle},
  time::{Duration, SystemTime},
};

use crossbeam_channel::{Receiver, Sender, TrySendError, bounded, select, tick, unbounded};
use rodio::{
  Decoder, DeviceSinkBuilder, Player, Source,
  buffer::SamplesBuffer,
  mixer::{Mixer, mixer},
  source::Zero,
};

use tg_service_async::EventSink;

use super::{
  AudioAsyncEvent, AudioCaptureId, AudioError, AudioErrorCode, AudioId, AudioPlaybackSnapshot,
  AudioPoolId, AudioSource,
};

const MAX_SOURCE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_DECODED_BYTES: usize = 256 * 1024 * 1024;
const MAX_CACHE_BYTES: usize = 512 * 1024 * 1024;
const PLAYBACK_POLL_INTERVAL: Duration = Duration::from_millis(20);
const CAPTURE_CHUNK_SAMPLES: usize = 2048;

/// An application request produced by audio interactions.
#[derive(Clone)]
pub(crate) enum AudioCommand {
  /// A playback-runtime command to load.
  Load {
    /// The identity of the owning pool.
    pool_id: AudioPoolId,
    /// The playback object identifier.
    audio_id: AudioId,
    /// The audio source carried by this audio command.
    source: AudioSource,
    /// The requested volume multiplier.
    volume: f32,
    /// The looped.
    looped: bool,
    /// The snapshot.
    snapshot: Arc<AudioPlaybackSnapshot>,
  },
  /// A playback-runtime command to remove.
  Remove {
    /// The playback object identifier.
    audio_id: AudioId,
  },
  /// A playback-runtime command to play.
  Play {
    /// The playback object identifier.
    audio_id: AudioId,
    /// The paused.
    paused: bool,
  },
  /// A playback-runtime command to pause.
  Pause {
    /// The playback object identifier.
    audio_id: AudioId,
  },
  /// A playback-runtime command to resume.
  Resume {
    /// The playback object identifier.
    audio_id: AudioId,
  },
  /// A playback-runtime command to stop.
  Stop {
    /// The playback object identifier.
    audio_id: AudioId,
  },
  /// A playback-runtime command to restart.
  Restart {
    /// The playback object identifier.
    audio_id: AudioId,
    /// The paused.
    paused: bool,
  },
  /// A playback-runtime command to set volume.
  SetVolume {
    /// The playback object identifier.
    audio_id: AudioId,
    /// The requested volume multiplier.
    volume: f32,
  },
  /// A playback-runtime command to set loop.
  SetLoop {
    /// The playback object identifier.
    audio_id: AudioId,
    /// The looped.
    looped: bool,
  },
  /// A playback-runtime command to seek.
  Seek {
    /// The playback object identifier.
    audio_id: AudioId,
    /// The position represented as a duration.
    position: Duration,
  },
  /// A playback-runtime command to start capture.
  StartCapture {
    /// The audio-capture identifier.
    capture_id: AudioCaptureId,
    /// The filesystem path to read, write, or resolve.
    path: PathBuf,
  },
  /// A playback-runtime command to pause capture.
  PauseCapture {
    /// The audio-capture identifier.
    capture_id: AudioCaptureId,
  },
  /// A playback-runtime command to resume capture.
  ResumeCapture {
    /// The audio-capture identifier.
    capture_id: AudioCaptureId,
  },
  /// A playback-runtime command to stop capture.
  StopCapture {
    /// The audio-capture identifier.
    capture_id: AudioCaptureId,
  },
  /// A playback-runtime command to stop all.
  StopAll,
  /// A playback-runtime command to release pool.
  ReleasePool {
    /// The identity of the owning pool.
    pool_id: AudioPoolId,
  },
  /// A playback-runtime command to clear cache.
  ClearCache,
  /// A playback-runtime command to shutdown.
  Shutdown,
}

#[derive(Default)]
struct CaptureTapState {
  generation: AtomicU64,
  enabled: AtomicBool,
  paused: AtomicBool,
}

enum CaptureWriterMessage {
  Begin {
    capture_id: AudioCaptureId,
    path: PathBuf,
    channels: u16,
    sample_rate: u32,
  },
  Samples {
    capture_id: AudioCaptureId,
    samples: Vec<f32>,
  },
  End {
    capture_id: AudioCaptureId,
  },
  Shutdown,
}

struct CaptureSource<S> {
  inner: S,
  state: Arc<CaptureTapState>,
  writer_tx: Sender<CaptureWriterMessage>,
  active: Option<AudioCaptureId>,
  samples: Vec<f32>,
  paused: bool,
}

impl<S> CaptureSource<S> {
  fn new(inner: S, state: Arc<CaptureTapState>, writer_tx: Sender<CaptureWriterMessage>) -> Self {
    Self {
      inner,
      state,
      writer_tx,
      active: None,
      samples: Vec::with_capacity(CAPTURE_CHUNK_SAMPLES),
      paused: false,
    }
  }

  fn flush_samples(&mut self) {
    let Some(capture_id) = self.active else {
      self.samples.clear();
      return;
    };
    if self.samples.is_empty() {
      return;
    }
    let samples = std::mem::replace(&mut self.samples, Vec::with_capacity(CAPTURE_CHUNK_SAMPLES));
    let _ = self.writer_tx.send(CaptureWriterMessage::Samples {
      capture_id,
      samples,
    });
  }

  fn synchronize_state(&mut self) {
    let enabled = self.state.enabled.load(Ordering::Acquire);
    let current = enabled.then(|| AudioCaptureId(self.state.generation.load(Ordering::Acquire)));
    let paused = enabled && self.state.paused.load(Ordering::Acquire);
    if self.active != current {
      self.flush_samples();
      if let Some(capture_id) = self.active.take() {
        let _ = self
          .writer_tx
          .send(CaptureWriterMessage::End { capture_id });
      }
      self.active = current;
      self.paused = paused;
    } else if self.paused != paused {
      self.flush_samples();
      self.paused = paused;
    }
  }
}

impl<S: Source> Iterator for CaptureSource<S> {
  type Item = f32;

  fn next(&mut self) -> Option<Self::Item> {
    let sample = self.inner.next()?;
    self.synchronize_state();
    if self.active.is_some() && !self.paused {
      self.samples.push(sample);
      if self.samples.len() >= CAPTURE_CHUNK_SAMPLES {
        self.flush_samples();
      }
    }
    Some(sample)
  }
}

impl<S: Source> Source for CaptureSource<S> {
  fn current_span_len(&self) -> Option<usize> {
    self.inner.current_span_len()
  }

  fn channels(&self) -> NonZeroU16 {
    self.inner.channels()
  }

  fn sample_rate(&self) -> NonZeroU32 {
    self.inner.sample_rate()
  }

  fn total_duration(&self) -> Option<Duration> {
    self.inner.total_duration()
  }
}

impl<S> Drop for CaptureSource<S> {
  fn drop(&mut self) {
    self.flush_samples();
    if let Some(capture_id) = self.active.take() {
      let _ = self
        .writer_tx
        .send(CaptureWriterMessage::End { capture_id });
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CacheKey {
  path: PathBuf,
  len: u64,
  modified: Option<SystemTime>,
}

struct DecodedAudio {
  samples: SamplesBuffer,
  duration: Duration,
  byte_len: usize,
}

enum CacheEntry {
  Loading {
    waiters: Vec<AudioId>,
  },
  Ready {
    audio: Arc<DecodedAudio>,
    last_used: u64,
  },
}

struct RuntimeInstance {
  pool_id: AudioPoolId,
  key: Option<CacheKey>,
  decoded: Option<Arc<DecodedAudio>>,
  player: Option<Player>,
  volume: f32,
  looped: bool,
  pending_play: bool,
  pending_paused: bool,
  pending_seek: Option<Duration>,
  stopped_while_loading: bool,
  snapshot: Arc<AudioPlaybackSnapshot>,
}

struct DecodeRequest {
  key: CacheKey,
}

struct DecodeResult {
  key: CacheKey,
  result: Result<DecodedAudio, AudioError>,
}

/// The audio runtime representation used by this module.
pub(crate) struct AudioRuntime {
  command_tx: Sender<AudioCommand>,
  control_thread: Option<JoinHandle<()>>,
}

impl AudioRuntime {
  /// Create an audio runtime initialized from `event_tx`.
  pub(crate) fn new(event_tx: EventSink<AudioAsyncEvent>) -> Self {
    let (command_tx, command_rx) = unbounded();
    let control_thread = thread::Builder::new()
      .name("audio-runtime".to_string())
      .spawn(move || run_audio_runtime(command_rx, event_tx))
      .ok();
    Self {
      command_tx,
      control_thread,
    }
  }

  /// Return the current command sender.
  pub(crate) fn command_sender(&self) -> Sender<AudioCommand> {
    self.command_tx.clone()
  }

  /// Send the supplied event or command to the owning runtime channel.
  ///
  /// # Errors
  ///
  /// Return `RuntimeClosed` when the audio command channel is disconnected.
  pub(crate) fn send(&self, command: AudioCommand) -> Result<(), AudioError> {
    self
      .command_tx
      .send(command)
      .map_err(|_| AudioError::sanitized(AudioErrorCode::RuntimeClosed))
  }

  /// Stop audio work and release its owned runtime resources.
  pub(crate) fn shutdown(&mut self) {
    let _ = self.command_tx.send(AudioCommand::Shutdown);
    if let Some(thread) = self.control_thread.take() {
      let _ = thread.join();
    }
  }
}

impl Drop for AudioRuntime {
  fn drop(&mut self) {
    if std::thread::panicking() {
      let _ = self.command_tx.send(AudioCommand::Shutdown);
      self.control_thread.take();
    } else {
      self.shutdown();
    }
  }
}

fn run_audio_runtime(command_rx: Receiver<AudioCommand>, event_tx: EventSink<AudioAsyncEvent>) {
  let (decode_tx, decode_rx) = bounded::<DecodeRequest>(8);
  let (decoded_tx, decoded_rx) = unbounded::<DecodeResult>();
  let (capture_tx, capture_rx) = unbounded::<CaptureWriterMessage>();
  let decoder_thread = thread::Builder::new()
    .name("audio-decoder".to_string())
    .spawn(move || run_decoder(decode_rx, decoded_tx))
    .ok();
  let capture_event_tx = event_tx.clone();
  let capture_thread = thread::Builder::new()
    .name("audio-capture-writer".to_string())
    .spawn(move || run_capture_writer(capture_rx, capture_event_tx))
    .ok();

  let stream_error_reported = Arc::new(AtomicBool::new(false));
  let stream_error_event_tx = event_tx.clone();
  let callback_error_reported = stream_error_reported.clone();
  let mut output = DeviceSinkBuilder::from_default_device()
    .map(|builder| {
      builder.with_error_callback(move |_error| {
        report_backend_failure_once(&stream_error_event_tx, &callback_error_reported);
      })
    })
    .and_then(DeviceSinkBuilder::open_stream)
    .ok();
  let capture_state = Arc::new(CaptureTapState::default());
  let mut output_player = None;
  let playback_mixer = output.as_ref().map(|output| {
    let channels = output.config().channel_count();
    let sample_rate = output.config().sample_rate();
    let (playback_mixer, playback_source) = mixer(channels, sample_rate);
    playback_mixer.add(Zero::new(channels, sample_rate));
    let player = Player::connect_new(output.mixer());
    player.append(CaptureSource::new(
      playback_source,
      capture_state.clone(),
      capture_tx.clone(),
    ));
    output_player = Some(player);
    playback_mixer
  });
  if let Some(output) = output.as_mut() {
    output.log_on_drop(false);
  } else {
    send_event(
      &event_tx,
      AudioAsyncEvent::BackendFailed {
        error: AudioError::sanitized(AudioErrorCode::BackendUnavailable),
      },
    );
  }

  let poll = tick(PLAYBACK_POLL_INTERVAL);
  let mut instances = HashMap::<AudioId, RuntimeInstance>::new();
  let mut cache = HashMap::<CacheKey, CacheEntry>::new();
  let mut cache_clock = 0_u64;

  loop {
    select! {
      recv(command_rx) -> command => {
        let Ok(command) = command else { break };
        if matches!(command, AudioCommand::Shutdown) {
          break;
        }
        handle_command(
          command,
          &event_tx,
          playback_mixer.as_ref(),
          output.as_ref().map(|output| {
            (
              output.config().channel_count().get(),
              output.config().sample_rate().get(),
            )
          }),
          &capture_state,
          &capture_tx,
          &decode_tx,
          &mut instances,
          &mut cache,
          &mut cache_clock,
        );
      }
      recv(decoded_rx) -> result => {
        if let Ok(result) = result {
          handle_decode_result(
            result,
            &event_tx,
            playback_mixer.as_ref(),
            &mut instances,
            &mut cache,
            &mut cache_clock,
          );
        }
      }
      recv(poll) -> _ => {
        poll_players(&event_tx, playback_mixer.as_ref(), &mut instances);
      }
    }
  }

  for instance in instances.values_mut() {
    if let Some(player) = instance.player.take() {
      player.stop();
    }
  }
  instances.clear();
  cache.clear();
  capture_state.enabled.store(false, Ordering::Release);
  drop(output_player);
  let _ = capture_tx.send(CaptureWriterMessage::Shutdown);
  drop(decode_tx);
  if let Some(thread) = decoder_thread {
    let _ = thread.join();
  }
  if let Some(thread) = capture_thread {
    let _ = thread.join();
  }
  drop(output);
}

#[allow(clippy::too_many_arguments)]
fn handle_command(
  command: AudioCommand,
  event_tx: &EventSink<AudioAsyncEvent>,
  playback_mixer: Option<&Mixer>,
  output_config: Option<(u16, u32)>,
  capture_state: &Arc<CaptureTapState>,
  capture_tx: &Sender<CaptureWriterMessage>,
  decode_tx: &Sender<DecodeRequest>,
  instances: &mut HashMap<AudioId, RuntimeInstance>,
  cache: &mut HashMap<CacheKey, CacheEntry>,
  cache_clock: &mut u64,
) {
  match command {
    AudioCommand::Load {
      pool_id,
      audio_id,
      source,
      volume,
      looped,
      snapshot,
    } => {
      let key = match cache_key(&source) {
        Ok(key) => key,
        Err(error) => {
          send_object_error(event_tx, pool_id, audio_id, error);
          return;
        }
      };
      let mut instance = RuntimeInstance {
        pool_id,
        key: Some(key.clone()),
        decoded: None,
        player: None,
        volume,
        looped,
        pending_play: false,
        pending_paused: false,
        pending_seek: None,
        stopped_while_loading: false,
        snapshot,
      };
      match cache.get_mut(&key) {
        Some(CacheEntry::Ready { audio, last_used }) => {
          *cache_clock = cache_clock.saturating_add(1);
          *last_used = *cache_clock;
          instance.decoded = Some(audio.clone());
          send_event(
            event_tx,
            AudioAsyncEvent::Ready {
              pool_id,
              audio_id,
              duration: audio.duration,
            },
          );
        }
        Some(CacheEntry::Loading { waiters }) => waiters.push(audio_id),
        None => {
          cache.insert(
            key.clone(),
            CacheEntry::Loading {
              waiters: vec![audio_id],
            },
          );
          if let Err(error) = decode_tx.try_send(DecodeRequest { key: key.clone() }) {
            cache.remove(&key);
            let code = match error {
              TrySendError::Full(_) => AudioErrorCode::Internal,
              TrySendError::Disconnected(_) => AudioErrorCode::RuntimeClosed,
            };
            send_object_error(event_tx, pool_id, audio_id, AudioError::sanitized(code));
          }
        }
      }
      instances.insert(audio_id, instance);
    }
    AudioCommand::Remove { audio_id } => {
      if let Some(mut instance) = instances.remove(&audio_id)
        && let Some(player) = instance.player.take()
      {
        player.stop();
      }
    }
    AudioCommand::Play { audio_id, paused } => {
      let Some(instance) = instances.get_mut(&audio_id) else {
        return;
      };
      instance.stopped_while_loading = false;
      instance.pending_play = true;
      instance.pending_paused = paused;
      if instance.decoded.is_some() {
        start_instance(event_tx, playback_mixer, audio_id, instance, true);
      }
    }
    AudioCommand::Pause { audio_id } => {
      if let Some(instance) = instances.get_mut(&audio_id) {
        instance.pending_paused = true;
        if let Some(player) = instance.player.as_ref() {
          player.pause();
          let position = player.get_pos();
          instance.snapshot.set_position(position);
          send_event(
            event_tx,
            AudioAsyncEvent::Paused {
              pool_id: instance.pool_id,
              audio_id,
              position,
            },
          );
        }
      }
    }
    AudioCommand::Resume { audio_id } => {
      if let Some(instance) = instances.get_mut(&audio_id) {
        instance.pending_paused = false;
        if let Some(player) = instance.player.as_ref() {
          player.play();
          let position = player.get_pos();
          send_event(
            event_tx,
            AudioAsyncEvent::Resumed {
              pool_id: instance.pool_id,
              audio_id,
              position,
            },
          );
        }
      }
    }
    AudioCommand::Stop { audio_id } => stop_instance(event_tx, instances, audio_id, true),
    AudioCommand::Restart { audio_id, paused } => {
      if let Some(instance) = instances.get_mut(&audio_id) {
        if let Some(player) = instance.player.take() {
          player.stop();
        }
        instance.snapshot.set_position(Duration::ZERO);
        instance.pending_seek = None;
        instance.pending_play = true;
        instance.pending_paused = paused;
        instance.stopped_while_loading = false;
        if instance.decoded.is_some() {
          start_instance(event_tx, playback_mixer, audio_id, instance, true);
        }
      }
    }
    AudioCommand::SetVolume { audio_id, volume } => {
      if let Some(instance) = instances.get_mut(&audio_id) {
        instance.volume = volume;
        if let Some(player) = instance.player.as_ref() {
          player.set_volume(volume);
        }
      }
    }
    AudioCommand::SetLoop { audio_id, looped } => {
      if let Some(instance) = instances.get_mut(&audio_id) {
        instance.looped = looped;
      }
    }
    AudioCommand::Seek { audio_id, position } => {
      if let Some(instance) = instances.get_mut(&audio_id) {
        instance.pending_seek = Some(position);
        if let Some(player) = instance.player.as_ref()
          && player.try_seek(position).is_ok()
        {
          instance.snapshot.set_position(position);
          instance.pending_seek = None;
        }
      }
    }
    AudioCommand::StartCapture { capture_id, path } => {
      let Some((channels, sample_rate)) = output_config else {
        send_event(
          event_tx,
          AudioAsyncEvent::CaptureFailed {
            capture_id,
            path,
            error: AudioError::sanitized(AudioErrorCode::BackendUnavailable),
          },
        );
        return;
      };
      let _ = capture_tx.send(CaptureWriterMessage::Begin {
        capture_id,
        path,
        channels,
        sample_rate,
      });
      capture_state
        .generation
        .store(capture_id.0, Ordering::Release);
      capture_state.paused.store(false, Ordering::Release);
      capture_state.enabled.store(true, Ordering::Release);
    }
    AudioCommand::PauseCapture { capture_id } => {
      if capture_state.enabled.load(Ordering::Acquire)
        && capture_state.generation.load(Ordering::Acquire) == capture_id.0
      {
        capture_state.paused.store(true, Ordering::Release);
      }
    }
    AudioCommand::ResumeCapture { capture_id } => {
      if capture_state.enabled.load(Ordering::Acquire)
        && capture_state.generation.load(Ordering::Acquire) == capture_id.0
      {
        capture_state.paused.store(false, Ordering::Release);
      }
    }
    AudioCommand::StopCapture { capture_id } => {
      if capture_state.enabled.load(Ordering::Acquire)
        && capture_state.generation.load(Ordering::Acquire) == capture_id.0
      {
        capture_state.enabled.store(false, Ordering::Release);
      }
    }
    AudioCommand::StopAll => {
      let ids: Vec<AudioId> = instances.keys().copied().collect();
      for id in ids {
        stop_instance(event_tx, instances, id, true);
      }
    }
    AudioCommand::ReleasePool { pool_id } => {
      let ids: Vec<AudioId> = instances
        .iter()
        .filter_map(|(id, instance)| (instance.pool_id == pool_id).then_some(*id))
        .collect();
      for id in ids {
        if let Some(mut instance) = instances.remove(&id)
          && let Some(player) = instance.player.take()
        {
          player.stop();
        }
      }
      clear_unreferenced_cache(cache);
    }
    AudioCommand::ClearCache => clear_unreferenced_cache(cache),
    AudioCommand::Shutdown => {}
  }
}

fn handle_decode_result(
  result: DecodeResult,
  event_tx: &EventSink<AudioAsyncEvent>,
  playback_mixer: Option<&Mixer>,
  instances: &mut HashMap<AudioId, RuntimeInstance>,
  cache: &mut HashMap<CacheKey, CacheEntry>,
  cache_clock: &mut u64,
) {
  let waiters = match cache.remove(&result.key) {
    Some(CacheEntry::Loading { waiters }) => waiters,
    Some(entry @ CacheEntry::Ready { .. }) => {
      cache.insert(result.key, entry);
      return;
    }
    None => return,
  };

  match result.result {
    Ok(decoded) => {
      if !reserve_cache_space(cache, decoded.byte_len) {
        let error = AudioError::sanitized(AudioErrorCode::TooLarge);
        for audio_id in waiters {
          if let Some(instance) = instances.get(&audio_id) {
            send_object_error(event_tx, instance.pool_id, audio_id, error.clone());
          }
        }
        return;
      }
      let decoded = Arc::new(decoded);
      *cache_clock = cache_clock.saturating_add(1);
      cache.insert(
        result.key.clone(),
        CacheEntry::Ready {
          audio: decoded.clone(),
          last_used: *cache_clock,
        },
      );
      for audio_id in waiters {
        let Some(instance) = instances.get_mut(&audio_id) else {
          continue;
        };
        if instance.key.as_ref() != Some(&result.key) {
          continue;
        }
        instance.decoded = Some(decoded.clone());
        send_event(
          event_tx,
          AudioAsyncEvent::Ready {
            pool_id: instance.pool_id,
            audio_id,
            duration: decoded.duration,
          },
        );
        if instance.pending_play && !instance.stopped_while_loading {
          start_instance(event_tx, playback_mixer, audio_id, instance, true);
        }
      }
    }
    Err(error) => {
      for audio_id in waiters {
        if let Some(instance) = instances.get(&audio_id) {
          send_object_error(event_tx, instance.pool_id, audio_id, error.clone());
        }
      }
    }
  }
}

fn start_instance(
  event_tx: &EventSink<AudioAsyncEvent>,
  playback_mixer: Option<&Mixer>,
  audio_id: AudioId,
  instance: &mut RuntimeInstance,
  emit_started: bool,
) {
  let Some(playback_mixer) = playback_mixer else {
    instance.pending_play = false;
    send_object_error(
      event_tx,
      instance.pool_id,
      audio_id,
      AudioError::sanitized(AudioErrorCode::BackendUnavailable),
    );
    return;
  };
  let Some(decoded) = instance.decoded.as_ref() else {
    return;
  };
  if let Some(player) = instance.player.take() {
    player.stop();
  }
  let player = Player::connect_new(playback_mixer);
  player.set_volume(instance.volume);
  player.append(decoded.samples.clone());
  let mut start_position = instance
    .pending_seek
    .take()
    .unwrap_or(Duration::ZERO)
    .min(decoded.duration);
  if !start_position.is_zero() && player.try_seek(start_position).is_err() {
    start_position = Duration::ZERO;
  }
  if instance.pending_paused {
    player.pause();
  }
  instance.snapshot.set_position(start_position);
  instance.pending_play = false;
  if emit_started {
    send_event(
      event_tx,
      AudioAsyncEvent::Started {
        pool_id: instance.pool_id,
        audio_id,
        position: start_position,
      },
    );
    if instance.pending_paused {
      send_event(
        event_tx,
        AudioAsyncEvent::Paused {
          pool_id: instance.pool_id,
          audio_id,
          position: start_position,
        },
      );
    }
  }
  instance.player = Some(player);
}

fn stop_instance(
  event_tx: &EventSink<AudioAsyncEvent>,
  instances: &mut HashMap<AudioId, RuntimeInstance>,
  audio_id: AudioId,
  emit_event: bool,
) {
  let Some(instance) = instances.get_mut(&audio_id) else {
    return;
  };
  if let Some(player) = instance.player.take() {
    player.stop();
  }
  instance.pending_play = false;
  instance.pending_seek = Some(Duration::ZERO);
  instance.stopped_while_loading = true;
  instance.snapshot.set_position(Duration::ZERO);
  if emit_event {
    send_event(
      event_tx,
      AudioAsyncEvent::Stopped {
        pool_id: instance.pool_id,
        audio_id,
      },
    );
  }
}

fn poll_players(
  event_tx: &EventSink<AudioAsyncEvent>,
  playback_mixer: Option<&Mixer>,
  instances: &mut HashMap<AudioId, RuntimeInstance>,
) {
  for (audio_id, instance) in instances.iter_mut() {
    let Some(player) = instance.player.as_ref() else {
      continue;
    };
    let position = player.get_pos();
    instance.snapshot.set_position(position);
    if !player.empty() {
      continue;
    }
    instance.player = None;
    let duration = instance
      .decoded
      .as_ref()
      .map(|audio| audio.duration)
      .unwrap_or(position);
    if instance.looped {
      start_instance(event_tx, playback_mixer, *audio_id, instance, false);
    } else {
      instance.snapshot.set_position(duration);
      send_event(
        event_tx,
        AudioAsyncEvent::Finished {
          pool_id: instance.pool_id,
          audio_id: *audio_id,
          duration,
        },
      );
    }
  }
}

struct CaptureFile {
  path: PathBuf,
  temporary_path: PathBuf,
  writer: Option<hound::WavWriter<BufWriter<File>>>,
  sample_rate: u32,
  channels: u16,
  samples_written: u64,
}

fn run_capture_writer(
  messages: Receiver<CaptureWriterMessage>,
  event_tx: EventSink<AudioAsyncEvent>,
) {
  let mut captures = HashMap::<AudioCaptureId, CaptureFile>::new();
  for message in messages {
    match message {
      CaptureWriterMessage::Begin {
        capture_id,
        path,
        channels,
        sample_rate,
      } => {
        let temporary_path = capture_temporary_path(&path);
        let result = path
          .parent()
          .map(fs::create_dir_all)
          .transpose()
          .and_then(|_| {
            let _ = fs::remove_file(&temporary_path);
            hound::WavWriter::create(
              &temporary_path,
              hound::WavSpec {
                channels,
                sample_rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
              },
            )
            .map_err(std::io::Error::other)
          });
        match result {
          Ok(writer) => {
            captures.insert(
              capture_id,
              CaptureFile {
                path,
                temporary_path,
                writer: Some(writer),
                sample_rate,
                channels,
                samples_written: 0,
              },
            );
          }
          Err(error) => send_event(
            &event_tx,
            AudioAsyncEvent::CaptureFailed {
              capture_id,
              path,
              error: AudioError::new(AudioErrorCode::Internal, error.to_string()),
            },
          ),
        }
      }
      CaptureWriterMessage::Samples {
        capture_id,
        samples,
      } => {
        let failed: Option<String> = captures.get_mut(&capture_id).and_then(|capture| {
          let writer = capture.writer.as_mut()?;
          for sample in samples {
            let value = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
            if let Err(error) = writer.write_sample(value) {
              return Some(error.to_string());
            }
            capture.samples_written = capture.samples_written.saturating_add(1);
          }
          None
        });
        if let Some(error) = failed
          && let Some(capture) = captures.remove(&capture_id)
        {
          fail_capture(capture_id, capture, error, &event_tx);
        }
      }
      CaptureWriterMessage::End { capture_id } => {
        if let Some(capture) = captures.remove(&capture_id) {
          finish_capture(capture_id, capture, &event_tx);
        }
      }
      CaptureWriterMessage::Shutdown => break,
    }
  }
  for (capture_id, capture) in captures {
    finish_capture(capture_id, capture, &event_tx);
  }
}

fn finish_capture(
  capture_id: AudioCaptureId,
  mut capture: CaptureFile,
  event_tx: &EventSink<AudioAsyncEvent>,
) {
  let result = (|| {
    capture
      .writer
      .take()
      .expect("capture writer is present until finalization")
      .finalize()
      .map_err(|error| error.to_string())?;
    File::options()
      .read(true)
      .write(true)
      .open(&capture.temporary_path)
      .and_then(|file| file.sync_all())
      .map_err(|error| error.to_string())?;
    fs::rename(&capture.temporary_path, &capture.path).map_err(|error| error.to_string())
  })();
  match result {
    Ok(()) => {
      let frames = capture
        .samples_written
        .checked_div(u64::from(capture.channels))
        .unwrap_or(0);
      let duration = Duration::from_secs_f64(frames as f64 / f64::from(capture.sample_rate));
      send_event(
        event_tx,
        AudioAsyncEvent::CaptureSaved {
          capture_id,
          path: capture.path,
          sample_rate: capture.sample_rate,
          channels: capture.channels,
          duration,
        },
      );
    }
    Err(error) => fail_capture(capture_id, capture, error, event_tx),
  }
}

fn fail_capture(
  capture_id: AudioCaptureId,
  capture: CaptureFile,
  error: String,
  event_tx: &EventSink<AudioAsyncEvent>,
) {
  let _ = fs::remove_file(&capture.temporary_path);
  send_event(
    event_tx,
    AudioAsyncEvent::CaptureFailed {
      capture_id,
      path: capture.path,
      error: AudioError::new(AudioErrorCode::Internal, error),
    },
  );
}

fn capture_temporary_path(path: &std::path::Path) -> PathBuf {
  let extension = path
    .extension()
    .and_then(|extension| extension.to_str())
    .unwrap_or("wav");
  path.with_extension(format!("{extension}.part"))
}

fn run_decoder(request_rx: Receiver<DecodeRequest>, result_tx: Sender<DecodeResult>) {
  for request in request_rx {
    let result = decode_file(&request.key);
    if result_tx
      .send(DecodeResult {
        key: request.key,
        result,
      })
      .is_err()
    {
      break;
    }
  }
}

fn decode_file(key: &CacheKey) -> Result<DecodedAudio, AudioError> {
  let file = fs::File::open(&key.path).map_err(map_file_error)?;
  let decoder = Decoder::try_from(file).map_err(|_| {
    AudioError::sanitized(if supported_extension(&key.path) {
      AudioErrorCode::Decode
    } else {
      AudioErrorCode::Unsupported
    })
  })?;
  let channels = decoder.channels();
  let sample_rate = decoder.sample_rate();
  let max_samples = MAX_DECODED_BYTES / std::mem::size_of::<f32>();
  let mut samples = Vec::with_capacity(decoder.size_hint().1.unwrap_or_default().min(max_samples));
  for sample in decoder {
    if samples.len() >= max_samples {
      return Err(AudioError::sanitized(AudioErrorCode::TooLarge));
    }
    samples.push(sample);
  }
  if samples.is_empty() {
    return Err(AudioError::sanitized(AudioErrorCode::Decode));
  }
  let duration = duration_for_samples(samples.len(), channels, sample_rate);
  let byte_len = samples.len() * std::mem::size_of::<f32>();
  Ok(DecodedAudio {
    samples: SamplesBuffer::new(channels, sample_rate, samples),
    duration,
    byte_len,
  })
}

fn duration_for_samples(samples: usize, channels: NonZeroU16, sample_rate: NonZeroU32) -> Duration {
  Duration::from_secs_f64(samples as f64 / channels.get() as f64 / sample_rate.get() as f64)
}

fn cache_key(source: &AudioSource) -> Result<CacheKey, AudioError> {
  let path = source.path();
  let metadata = fs::metadata(path).map_err(map_file_error)?;
  if !metadata.is_file() {
    return Err(AudioError::sanitized(AudioErrorCode::NotFound));
  }
  if metadata.len() > MAX_SOURCE_BYTES {
    return Err(AudioError::sanitized(AudioErrorCode::TooLarge));
  }
  Ok(CacheKey {
    path: path.to_path_buf(),
    len: metadata.len(),
    modified: metadata.modified().ok(),
  })
}

fn map_file_error(error: std::io::Error) -> AudioError {
  let code = match error.kind() {
    std::io::ErrorKind::NotFound => AudioErrorCode::NotFound,
    std::io::ErrorKind::PermissionDenied => AudioErrorCode::PermissionDenied,
    _ => AudioErrorCode::Internal,
  };
  AudioError::sanitized(code)
}

fn supported_extension(path: &std::path::Path) -> bool {
  path
    .extension()
    .and_then(|extension| extension.to_str())
    .is_some_and(|extension| {
      matches!(
        extension.to_ascii_lowercase().as_str(),
        "wav" | "mp3" | "flac" | "ogg" | "oga" | "mp4" | "m4a" | "aac"
      )
    })
}

fn reserve_cache_space(cache: &mut HashMap<CacheKey, CacheEntry>, required: usize) -> bool {
  if required > MAX_CACHE_BYTES {
    return false;
  }
  loop {
    let used = cache
      .values()
      .filter_map(|entry| match entry {
        CacheEntry::Ready { audio, .. } => Some(audio.byte_len),
        CacheEntry::Loading { .. } => None,
      })
      .sum::<usize>();
    if used.saturating_add(required) <= MAX_CACHE_BYTES {
      return true;
    }
    let candidate = cache
      .iter()
      .filter_map(|(key, entry)| match entry {
        CacheEntry::Ready { audio, last_used } if Arc::strong_count(audio) == 1 => {
          Some((key.clone(), *last_used))
        }
        _ => None,
      })
      .min_by_key(|(_, last_used)| *last_used)
      .map(|(key, _)| key);
    let Some(candidate) = candidate else {
      return false;
    };
    cache.remove(&candidate);
  }
}

fn clear_unreferenced_cache(cache: &mut HashMap<CacheKey, CacheEntry>) {
  cache.retain(|_, entry| match entry {
    CacheEntry::Loading { .. } => true,
    CacheEntry::Ready { audio, .. } => Arc::strong_count(audio) > 1,
  });
}

fn send_object_error(
  event_tx: &EventSink<AudioAsyncEvent>,
  pool_id: AudioPoolId,
  audio_id: AudioId,
  error: AudioError,
) {
  send_event(
    event_tx,
    AudioAsyncEvent::Failed {
      pool_id,
      audio_id,
      error,
    },
  );
}

fn report_backend_failure_once(
  event_tx: &EventSink<AudioAsyncEvent>,
  already_reported: &AtomicBool,
) {
  if already_reported.swap(true, Ordering::AcqRel) {
    return;
  }
  send_event(
    event_tx,
    AudioAsyncEvent::BackendFailed {
      error: AudioError::sanitized(AudioErrorCode::BackendUnavailable),
    },
  );
}

fn send_event(event_tx: &EventSink<AudioAsyncEvent>, event: AudioAsyncEvent) {
  event_tx.send(event);
}

#[cfg(test)]
mod tests {
  use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
  };

  use super::*;

  fn temporary_path(extension: &str) -> PathBuf {
    static NEXT_FILE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let sequence = NEXT_FILE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
      "tui-game-audio-{}-{nonce}-{sequence}.{extension}",
      std::process::id()
    ))
  }

  fn write_test_wav(path: &Path) {
    let samples = [0_i16, 1_000, -1_000, 2_000, -2_000, 0, 500, -500];
    let data_size = (samples.len() * std::mem::size_of::<i16>()) as u32;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&8_000_u32.to_le_bytes());
    bytes.extend_from_slice(&16_000_u32.to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
      bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(path, bytes).unwrap();
  }

  #[test]
  fn wav_decoding_produces_shared_pcm_metadata() {
    let path = temporary_path("wav");
    write_test_wav(&path);
    let key = CacheKey {
      path: path.clone(),
      len: fs::metadata(&path).unwrap().len(),
      modified: fs::metadata(&path).unwrap().modified().ok(),
    };
    let decoded = decode_file(&key).unwrap();
    assert!(decoded.byte_len > 0);
    assert_eq!(decoded.duration, Duration::from_millis(1));
    fs::remove_file(path).unwrap();
  }

  #[test]
  fn unsupported_files_return_sanitized_errors() {
    let path = temporary_path("bin");
    fs::write(&path, b"not audio").unwrap();
    let key = CacheKey {
      path: path.clone(),
      len: fs::metadata(&path).unwrap().len(),
      modified: None,
    };
    let error = match decode_file(&key) {
      Ok(_) => panic!("unsupported test data decoded successfully"),
      Err(error) => error,
    };
    assert_eq!(error.code, AudioErrorCode::Unsupported);
    assert!(!error.message.contains(path.to_string_lossy().as_ref()));
    fs::remove_file(path).unwrap();
  }

  #[test]
  fn capture_writer_commits_a_valid_wav_atomically() {
    let path = temporary_path("wav");
    let temporary = capture_temporary_path(&path);
    let (message_tx, message_rx) = unbounded();
    let (event_tx, event_rx) = unbounded::<AudioAsyncEvent>();
    let capture_id = AudioCaptureId(7);
    message_tx
      .send(CaptureWriterMessage::Begin {
        capture_id,
        path: path.clone(),
        channels: 2,
        sample_rate: 8_000,
      })
      .unwrap();
    message_tx
      .send(CaptureWriterMessage::Samples {
        capture_id,
        samples: vec![0.0, 0.5, -0.5, 1.0],
      })
      .unwrap();
    message_tx
      .send(CaptureWriterMessage::End { capture_id })
      .unwrap();
    message_tx.send(CaptureWriterMessage::Shutdown).unwrap();
    drop(message_tx);

    run_capture_writer(message_rx, EventSink::new(event_tx));

    assert!(path.is_file());
    assert!(!temporary.exists());
    let reader = hound::WavReader::open(&path).unwrap();
    assert_eq!(reader.spec().channels, 2);
    assert_eq!(reader.spec().sample_rate, 8_000);
    assert_eq!(reader.len(), 4);
    assert!(matches!(
      event_rx.recv().unwrap(),
      AudioAsyncEvent::CaptureSaved {
        capture_id: AudioCaptureId(7),
        channels: 2,
        sample_rate: 8_000,
        ..
      }
    ));
    fs::remove_file(path).unwrap();
  }

  #[test]
  fn cache_never_evicts_audio_with_live_instance_references() {
    let key = CacheKey {
      path: PathBuf::from("one.wav"),
      len: 1,
      modified: None,
    };
    let decoded = Arc::new(DecodedAudio {
      samples: SamplesBuffer::new(
        NonZeroU16::new(1).unwrap(),
        NonZeroU32::new(8_000).unwrap(),
        vec![0.0],
      ),
      duration: Duration::from_millis(1),
      byte_len: MAX_CACHE_BYTES,
    });
    let live_reference = decoded.clone();
    let mut cache = HashMap::from([(
      key.clone(),
      CacheEntry::Ready {
        audio: decoded,
        last_used: 1,
      },
    )]);
    assert!(!reserve_cache_space(&mut cache, 1));
    assert!(cache.contains_key(&key));
    drop(live_reference);
    assert!(reserve_cache_space(&mut cache, 1));
    assert!(!cache.contains_key(&key));
  }

  #[test]
  fn backend_stream_errors_are_reported_once_through_engine_events() {
    let (event_tx, event_rx) = unbounded::<AudioAsyncEvent>();
    let already_reported = AtomicBool::new(false);

    let event_tx = EventSink::new(event_tx);
    report_backend_failure_once(&event_tx, &already_reported);
    report_backend_failure_once(&event_tx, &already_reported);

    assert!(matches!(
      event_rx.recv().unwrap(),
      AudioAsyncEvent::BackendFailed { error }
        if error.code == AudioErrorCode::BackendUnavailable
    ));
    assert!(event_rx.try_recv().is_err());
  }
}
