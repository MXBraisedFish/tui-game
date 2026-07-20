mod easing;
mod effect;
mod pool;
mod service;
mod types;

pub use effect::CharacterEffectService;
pub use service::{AnimationService, AnimationTargetRouter};
pub use types::{
  AnimationBinding, AnimationCallbackId, AnimationCallbackRequest, AnimationClip, AnimationClock,
  AnimationColor, AnimationEasing, AnimationEndMode, AnimationError, AnimationEvent,
  AnimationEventKind, AnimationHandle, AnimationId, AnimationInterpolation, AnimationKeyframe,
  AnimationMarker, AnimationOwner, AnimationPlaybackOptions, AnimationProperty,
  AnimationRepeatCount, AnimationRepeatMode, AnimationRepeatOptions, AnimationSource,
  AnimationTarget, AnimationTrack, AnimationUpdate, AnimationValue, AnimationValueId,
  AnimationValueKind, AnimationWrite, AnimationWriteOperation, CellEffectId, CharacterFrame,
  EffectParameterId, GameInstanceId, GameObjectRef, PlaybackDirection, PlaybackState,
  TweenDefinition, UiObjectKind, UiObjectRef, UiPoolId,
};

pub(crate) use pool::{AnimationPool, AnimationValuePool, CharacterEffectPool};

#[cfg(test)]
mod tests;
