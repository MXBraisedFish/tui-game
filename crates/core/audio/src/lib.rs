//! Audio data model: pool/object/capture ids, playback state, errors, resolved sources and
//! async playback events.

use std::{
  path::{Path, PathBuf},
  time::Duration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioPoolId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioCaptureId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioTypeId {
  pub pool_id: AudioPoolId,
  pub index: u32,
  pub generation: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioId {
  pub pool_id: AudioPoolId,
  pub index: u32,
  pub generation: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioState {
  Created,
  Loading,
  Ready,
  Playing,
  Paused,
  Stopped,
  Finished,
  Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioErrorCode {
  InvalidId,
  InvalidVolume,
  InvalidState,
  TypeInUse,
  InvalidPath,
  NotFound,
  PermissionDenied,
  TooLarge,
  Unsupported,
  Decode,
  BackendUnavailable,
  RuntimeClosed,
  Internal,
}

impl AudioErrorCode {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioError {
  pub code: AudioErrorCode,
  pub message: String,
}

impl AudioError {
  pub fn new(code: AudioErrorCode, message: impl Into<String>) -> Self {
    Self {
      code,
      message: message.into(),
    }
  }

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

/// A path that has already been canonicalized and checked by PackageService or StorageService.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResolvedAudioFile {
  path: PathBuf,
}

impl ResolvedAudioFile {
  pub fn new(path: PathBuf) -> Self {
    Self { path }
  }

  pub fn path(&self) -> &Path {
    &self.path
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AudioSource {
  File(ResolvedAudioFile),
}

impl AudioSource {
  pub fn path(&self) -> &Path {
    match self {
      Self::File(file) => file.path(),
    }
  }
}

#[derive(Clone, Debug)]
pub enum AudioAsyncEvent {
  Ready {
    pool_id: AudioPoolId,
    audio_id: AudioId,
    duration: Duration,
  },
  Started {
    pool_id: AudioPoolId,
    audio_id: AudioId,
    position: Duration,
  },
  Paused {
    pool_id: AudioPoolId,
    audio_id: AudioId,
    position: Duration,
  },
  Resumed {
    pool_id: AudioPoolId,
    audio_id: AudioId,
    position: Duration,
  },
  Stopped {
    pool_id: AudioPoolId,
    audio_id: AudioId,
  },
  Finished {
    pool_id: AudioPoolId,
    audio_id: AudioId,
    duration: Duration,
  },
  Failed {
    pool_id: AudioPoolId,
    audio_id: AudioId,
    error: AudioError,
  },
  BackendFailed {
    error: AudioError,
  },
  CaptureSaved {
    capture_id: AudioCaptureId,
    path: PathBuf,
    sample_rate: u32,
    channels: u16,
    duration: Duration,
  },
  CaptureFailed {
    capture_id: AudioCaptureId,
    path: PathBuf,
    error: AudioError,
  },
}

impl AudioAsyncEvent {
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
