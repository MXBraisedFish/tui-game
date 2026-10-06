//! Playback identities, resolved audio sources, states, and asynchronous audio events.
//!
//! # Examples
//!
//! ```rust
//! use tg_core_audio::{AudioError, AudioErrorCode};
//!
//! let error = AudioError::sanitized(AudioErrorCode::InvalidId);
//! assert_eq!(error.code, AudioErrorCode::InvalidId);
//! assert!(!error.message.is_empty());
//! ```

use std::{
  path::{Path, PathBuf},
  time::Duration,
};

/// The identity of audio pool within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioPoolId(
  /// The wrapped u64 value.
  pub u64,
);

/// The identity of audio capture within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioCaptureId(
  /// The wrapped u64 value.
  pub u64,
);

/// The identity of audio type within its owning pool or session.
///
/// # Fields
///
/// * `pool_id` - The identity of the owning pool.
/// * `index` - The slot or sequence index.
/// * `generation` - The generation used to distinguish live handles from reused identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioTypeId {
  /// The identity of the owning pool.
  pub pool_id: AudioPoolId,
  /// The slot or sequence index.
  pub index: u32,
  /// The generation used to distinguish live handles from reused identifiers.
  pub generation: u32,
}

/// The identity of audio within its owning pool or session.
///
/// # Fields
///
/// * `pool_id` - The identity of the owning pool.
/// * `index` - The slot or sequence index.
/// * `generation` - The generation used to distinguish live handles from reused identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioId {
  /// The identity of the owning pool.
  pub pool_id: AudioPoolId,
  /// The slot or sequence index.
  pub index: u32,
  /// The generation used to distinguish live handles from reused identifiers.
  pub generation: u32,
}

/// The loading and playback lifecycle of an audio object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioState {
  /// The operation is created.
  Created,
  /// The operation is loading.
  Loading,
  /// The operation is ready.
  Ready,
  /// The operation is playing.
  Playing,
  /// The operation is paused.
  Paused,
  /// The operation is stopped.
  Stopped,
  /// The operation is finished.
  Finished,
  /// The operation is failed.
  Failed,
}

/// Failures reported by audio operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioErrorCode {
  /// The invalid id failure condition.
  InvalidId,
  /// The invalid volume failure condition.
  InvalidVolume,
  /// The invalid state failure condition.
  InvalidState,
  /// The type in use failure condition.
  TypeInUse,
  /// The invalid path failure condition.
  InvalidPath,
  /// The not found failure condition.
  NotFound,
  /// The permission denied failure condition.
  PermissionDenied,
  /// The too large failure condition.
  TooLarge,
  /// The unsupported failure condition.
  Unsupported,
  /// The decode failure condition.
  Decode,
  /// The backend unavailable failure condition.
  BackendUnavailable,
  /// The runtime closed failure condition.
  RuntimeClosed,
  /// The internal failure condition.
  Internal,
}

impl AudioErrorCode {
  /// Return the stable string key for this audio error code.
  pub fn as_str(self) -> &'static str {
    match self {
      Self::InvalidId => "invalid_id",
      Self::InvalidVolume => "invalid_volume",
      Self::InvalidState => "invalid_state",
      Self::TypeInUse => "type_in_use",
      Self::InvalidPath => "invalid_path",
      Self::NotFound => "not_found",
      Self::PermissionDenied => "permission_denied",
      Self::TooLarge => "too_large",
      Self::Unsupported => "unsupported",
      Self::Decode => "decode",
      Self::BackendUnavailable => "backend_unavailable",
      Self::RuntimeClosed => "runtime_closed",
      Self::Internal => "internal",
    }
  }
}

/// Failures reported by audio operations.
///
/// # Fields
///
/// * `code` - The stable error or language code.
/// * `message` - The diagnostic or display message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioError {
  /// The stable error or language code.
  pub code: AudioErrorCode,
  /// The diagnostic or display message.
  pub message: String,
}

impl AudioError {
  /// Create an audio failure with its stable code and diagnostic message.
  pub fn new(code: AudioErrorCode, message: impl Into<String>) -> Self {
    Self {
      code,
      message: message.into(),
    }
  }

  /// Create a stable public error message from its code without exposing internal diagnostic
  /// detail.
  pub fn sanitized(code: AudioErrorCode) -> Self {
    let message = match code {
      AudioErrorCode::InvalidId => "audio object does not exist",
      AudioErrorCode::InvalidVolume => "audio volume is invalid",
      AudioErrorCode::InvalidState => "audio operation is invalid for the current state",
      AudioErrorCode::TypeInUse => "audio type is still in use",
      AudioErrorCode::InvalidPath => "audio path is invalid",
      AudioErrorCode::NotFound => "audio resource was not found",
      AudioErrorCode::PermissionDenied => "audio resource is not permitted",
      AudioErrorCode::TooLarge => "audio resource exceeds its size limit",
      AudioErrorCode::Unsupported => "audio format is not supported",
      AudioErrorCode::Decode => "audio resource could not be decoded",
      AudioErrorCode::BackendUnavailable => "audio output is unavailable",
      AudioErrorCode::RuntimeClosed => "audio runtime is closed",
      AudioErrorCode::Internal => "internal audio operation failed",
    };
    Self::new(code, message)
  }
}

impl std::fmt::Display for AudioError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "{}: {}", self.code.as_str(), self.message)
  }
}

impl std::error::Error for AudioError {}

/// An audio file path already canonicalized and checked against its allowed asset root.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResolvedAudioFile {
  path: PathBuf,
}

impl ResolvedAudioFile {
  /// Wrap an already validated audio file path without performing additional filesystem checks.
  pub fn new(path: PathBuf) -> Self {
    Self { path }
  }

  /// Return the current path.
  pub fn path(&self) -> &Path {
    &self.path
  }
}

/// A validated source of audio playback data.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AudioSource {
  /// Content originating from file.
  File(ResolvedAudioFile),
}

impl AudioSource {
  /// Return the current path.
  pub fn path(&self) -> &Path {
    match self {
      Self::File(file) => file.path(),
    }
  }
}

/// Playback and capture outcomes emitted by the audio runtime.
#[derive(Clone, Debug)]
pub enum AudioAsyncEvent {
  /// A ready notification delivered to the owning consumer.
  Ready {
    /// The identity of the owning pool.
    pool_id: AudioPoolId,
    /// The playback object identifier.
    audio_id: AudioId,
    /// The duration represented as a duration.
    duration: Duration,
  },
  /// A started notification delivered to the owning consumer.
  Started {
    /// The identity of the owning pool.
    pool_id: AudioPoolId,
    /// The playback object identifier.
    audio_id: AudioId,
    /// The position represented as a duration.
    position: Duration,
  },
  /// The operation is paused.
  Paused {
    /// The identity of the owning pool.
    pool_id: AudioPoolId,
    /// The playback object identifier.
    audio_id: AudioId,
    /// The position represented as a duration.
    position: Duration,
  },
  /// A resumed notification delivered to the owning consumer.
  Resumed {
    /// The identity of the owning pool.
    pool_id: AudioPoolId,
    /// The playback object identifier.
    audio_id: AudioId,
    /// The position represented as a duration.
    position: Duration,
  },
  /// The operation is stopped.
  Stopped {
    /// The identity of the owning pool.
    pool_id: AudioPoolId,
    /// The playback object identifier.
    audio_id: AudioId,
  },
  /// The operation is finished.
  Finished {
    /// The identity of the owning pool.
    pool_id: AudioPoolId,
    /// The playback object identifier.
    audio_id: AudioId,
    /// The duration represented as a duration.
    duration: Duration,
  },
  /// A failed notification delivered to the owning consumer.
  Failed {
    /// The identity of the owning pool.
    pool_id: AudioPoolId,
    /// The playback object identifier.
    audio_id: AudioId,
    /// The error.
    error: AudioError,
  },
  /// A backend failed notification delivered to the owning consumer.
  BackendFailed {
    /// The error.
    error: AudioError,
  },
  /// A capture saved notification delivered to the owning consumer.
  CaptureSaved {
    /// The audio-capture identifier.
    capture_id: AudioCaptureId,
    /// The filesystem path to read, write, or resolve.
    path: PathBuf,
    /// The sample rate.
    sample_rate: u32,
    /// The channels.
    channels: u16,
    /// The duration represented as a duration.
    duration: Duration,
  },
  /// A capture failed notification delivered to the owning consumer.
  CaptureFailed {
    /// The audio-capture identifier.
    capture_id: AudioCaptureId,
    /// The filesystem path to read, write, or resolve.
    path: PathBuf,
    /// The error.
    error: AudioError,
  },
}

impl AudioAsyncEvent {
  /// Return the current pool id.
  pub fn pool_id(&self) -> Option<AudioPoolId> {
    match self {
      Self::Ready { pool_id, .. }
      | Self::Started { pool_id, .. }
      | Self::Paused { pool_id, .. }
      | Self::Resumed { pool_id, .. }
      | Self::Stopped { pool_id, .. }
      | Self::Finished { pool_id, .. }
      | Self::Failed { pool_id, .. } => Some(*pool_id),
      Self::BackendFailed { .. } | Self::CaptureSaved { .. } | Self::CaptureFailed { .. } => None,
    }
  }

  /// Return the current audio id.
  pub fn audio_id(&self) -> Option<AudioId> {
    match self {
      Self::Ready { audio_id, .. }
      | Self::Started { audio_id, .. }
      | Self::Paused { audio_id, .. }
      | Self::Resumed { audio_id, .. }
      | Self::Stopped { audio_id, .. }
      | Self::Finished { audio_id, .. }
      | Self::Failed { audio_id, .. } => Some(*audio_id),
      Self::BackendFailed { .. } | Self::CaptureSaved { .. } | Self::CaptureFailed { .. } => None,
    }
  }

  /// Report whether this audio async event has valid identity.
  pub fn has_valid_identity(&self) -> bool {
    match (self.pool_id(), self.audio_id()) {
      (Some(pool_id), Some(audio_id)) => pool_id == audio_id.pool_id,
      (None, None) => true,
      _ => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn events_must_carry_matching_pool_and_audio_ids() {
    let audio_id = AudioId {
      pool_id: AudioPoolId(1),
      index: 0,
      generation: 0,
    };
    let stopped = |pool_id| AudioAsyncEvent::Stopped { pool_id, audio_id };
    assert!(stopped(AudioPoolId(1)).has_valid_identity());
    assert!(!stopped(AudioPoolId(2)).has_valid_identity());
    let backend = AudioAsyncEvent::BackendFailed {
      error: AudioError::sanitized(AudioErrorCode::BackendUnavailable),
    };
    assert!(backend.has_valid_identity());
    assert_eq!(backend.audio_id(), None);
    assert_eq!(
      AudioError::sanitized(AudioErrorCode::NotFound).to_string(),
      "not_found: audio resource was not found"
    );
  }
}
