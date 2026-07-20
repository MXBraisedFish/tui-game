use std::time::Duration;

use crate::host_engine::services::widget::runtime_object::RuntimeObjectPool;

use super::easing;
use super::pool::{AnimatedValue, AnimationPlayback};
use super::{
  AnimationBinding, AnimationCallbackRequest, AnimationClock, AnimationEndMode, AnimationError,
  AnimationEvent, AnimationEventKind, AnimationHandle, AnimationId, AnimationInterpolation,
  AnimationOwner, AnimationProperty, AnimationRepeatCount, AnimationRepeatMode, AnimationSource,
  AnimationTarget, AnimationUpdate, AnimationValue, AnimationValueId, AnimationValueKind,
  AnimationWrite, AnimationWriteOperation, EffectParameterId, PlaybackDirection, PlaybackState,
};

pub trait AnimationTargetRouter {
  fn apply(&mut self, write: &AnimationWrite) -> Result<(), AnimationError>;
}

pub struct AnimationService;

impl AnimationService {
  pub fn new() -> Self {
    Self
  }

  pub fn play(
    &self,
    pool: &mut RuntimeObjectPool,
    owner: AnimationOwner,
    source: AnimationSource,
    bindings: Vec<AnimationBinding>,
    options: super::AnimationPlaybackOptions,
  ) -> Result<AnimationHandle, AnimationError> {
    self.validate(&source, &bindings, &options)?;
    self.validate_internal_targets(pool, &bindings)?;
    let auto_play = options.auto_play;
    let playback = AnimationPlayback::new(owner, source, bindings, options);
    let id = pool.animations.insert(playback);
    if auto_play {
      let playback = pool
        .animations
        .get(id)
        .expect("new playback must exist")
        .clone();
      self.push_event(pool, id, &playback, AnimationEventKind::Started, None);
    }
    Ok(AnimationHandle::new(id))
  }

  pub fn update(
    &self,
    pool: &mut RuntimeObjectPool,
    clock: AnimationClock,
    dt: Duration,
  ) -> AnimationUpdate {
    let mut output = AnimationUpdate::default();
    for id in pool.animations.ids() {
      let Some(mut playback) = pool.animations.get(id).cloned() else {
        continue;
      };
      if playback.clock != clock || playback.state != PlaybackState::Playing {
        continue;
      }

      let remaining = self.consume_delay(&mut playback, dt);
      if playback.delay_elapsed < playback_options_delay(&playback) {
        if let Some(stored) = pool.animations.get_mut(id) {
          *stored = playback;
        }
        continue;
      }

      let scaled = scale_duration(remaining, playback.speed);
      if scaled.is_zero() {
        self.sample_playback(id, &mut playback, &mut output.writes);
      } else {
        self.advance(pool, id, &mut playback, scaled, &mut output);
      }
      if let Some(stored) = pool.animations.get_mut(id) {
        *stored = playback;
      }
    }
    self.apply_internal_writes(pool, &output.writes);
    output
  }

  pub fn update_and_apply(
    &self,
    pool: &mut RuntimeObjectPool,
    clock: AnimationClock,
    dt: Duration,
    router: &mut impl AnimationTargetRouter,
  ) -> Result<AnimationUpdate, AnimationError> {
    let output = self.update(pool, clock, dt);
    for write in &output.writes {
      if !matches!(
        write.target,
        AnimationTarget::Value(_) | AnimationTarget::Effect(_)
      ) {
        if let Err(error) = router.apply(write) {
          if matches!(error, AnimationError::TargetNotFound(_)) {
            let cancelled = self.cancel(pool, AnimationHandle::new(write.animation_id))?;
            for cleanup in &cancelled.writes {
              if !matches!(
                cleanup.target,
                AnimationTarget::Value(_) | AnimationTarget::Effect(_)
              ) {
                let _ = router.apply(cleanup);
              }
            }
          }
          return Err(error);
        }
      }
    }
    Ok(output)
  }

  pub fn start(
    &self,
    pool: &mut RuntimeObjectPool,
    handle: AnimationHandle,
  ) -> Result<(), AnimationError> {
    let id = handle.id();
    let playback = pool
      .animations
      .get_mut(id)
      .ok_or(AnimationError::StaleAnimation)?;
    if matches!(
      playback.state,
      PlaybackState::Finished | PlaybackState::Cancelled
    ) {
      reset_playback_time(playback);
    }
    playback.state = PlaybackState::Playing;
    let playback = playback.clone();
    self.push_event(pool, id, &playback, AnimationEventKind::Started, None);
    Ok(())
  }

  pub fn pause(
    &self,
    pool: &mut RuntimeObjectPool,
    handle: AnimationHandle,
  ) -> Result<(), AnimationError> {
    let playback = pool
      .animations
      .get_mut(handle.id())
      .ok_or(AnimationError::StaleAnimation)?;
    if playback.state != PlaybackState::Playing {
      return Err(AnimationError::InvalidPlaybackState {
        expected: PlaybackState::Playing,
        actual: playback.state,
      });
    }
    playback.state = PlaybackState::Paused;
    Ok(())
  }

  pub fn resume(
    &self,
    pool: &mut RuntimeObjectPool,
    handle: AnimationHandle,
  ) -> Result<(), AnimationError> {
    let playback = pool
      .animations
      .get_mut(handle.id())
      .ok_or(AnimationError::StaleAnimation)?;
    if playback.state != PlaybackState::Paused {
      return Err(AnimationError::InvalidPlaybackState {
        expected: PlaybackState::Paused,
        actual: playback.state,
      });
    }
    playback.state = PlaybackState::Playing;
    Ok(())
  }

  pub fn cancel(
    &self,
    pool: &mut RuntimeObjectPool,
    handle: AnimationHandle,
  ) -> Result<AnimationUpdate, AnimationError> {
    let id = handle.id();
    let mut playback = pool
      .animations
      .get(id)
      .cloned()
      .ok_or(AnimationError::StaleAnimation)?;
    let mut output = AnimationUpdate::default();
    for (binding, value) in playback.bindings.iter().zip(&playback.current_values) {
      push_commit_and_clear(id, binding, value.clone(), &mut output.writes);
    }
    playback.state = PlaybackState::Cancelled;
    self.push_event(
      pool,
      id,
      &playback,
      AnimationEventKind::Cancelled,
      Some(&mut output.events),
    );
    *pool
      .animations
      .get_mut(id)
      .ok_or(AnimationError::StaleAnimation)? = playback;
    self.apply_internal_writes(pool, &output.writes);
    Ok(output)
  }

  pub fn finish(
    &self,
    pool: &mut RuntimeObjectPool,
    handle: AnimationHandle,
  ) -> Result<AnimationUpdate, AnimationError> {
    let id = handle.id();
    let mut playback = pool
      .animations
      .get(id)
      .cloned()
      .ok_or(AnimationError::StaleAnimation)?;
    playback.elapsed = playback.source.duration();
    let mut output = AnimationUpdate::default();
    self.sample_playback(id, &mut playback, &mut output.writes);
    self.complete(pool, id, &mut playback, &mut output);
    *pool
      .animations
      .get_mut(id)
      .ok_or(AnimationError::StaleAnimation)? = playback;
    self.apply_internal_writes(pool, &output.writes);
    Ok(output)
  }

  pub fn reset(
    &self,
    pool: &mut RuntimeObjectPool,
    handle: AnimationHandle,
  ) -> Result<AnimationUpdate, AnimationError> {
    let id = handle.id();
    let playback = pool
      .animations
      .get_mut(id)
      .ok_or(AnimationError::StaleAnimation)?;
    let mut output = AnimationUpdate::default();
    for binding in &playback.bindings {
      push_commit_and_clear(
        id,
        binding,
        binding.initial_value.clone(),
        &mut output.writes,
      );
    }
    reset_playback_time(playback);
    playback.state = PlaybackState::Idle;
    playback.current_values = playback
      .bindings
      .iter()
      .map(|binding| binding.initial_value.clone())
      .collect();
    pool.animations.events.retain(|event| event.id != id);
    pool
      .animations
      .callback_requests
      .retain(|request| request.event.id != id);
    self.apply_internal_writes(pool, &output.writes);
    Ok(output)
  }

  pub fn remove(
    &self,
    pool: &mut RuntimeObjectPool,
    handle: AnimationHandle,
  ) -> Result<AnimationUpdate, AnimationError> {
    let output = self.reset(pool, handle)?;
    pool
      .animations
      .remove(handle.id())
      .ok_or(AnimationError::StaleAnimation)?;
    Ok(output)
  }

  pub fn clear_owner(
    &self,
    pool: &mut RuntimeObjectPool,
    owner: AnimationOwner,
  ) -> AnimationUpdate {
    let mut output = AnimationUpdate::default();
    for id in pool.animations.ids_owned_by(owner) {
      if let Ok(removed) = self.remove(pool, AnimationHandle::new(id)) {
        output.writes.extend(removed.writes);
        output.events.extend(removed.events);
      }
    }
    output
  }

  pub fn state(&self, pool: &RuntimeObjectPool, handle: AnimationHandle) -> Option<PlaybackState> {
    Some(pool.animations.get(handle.id())?.state)
  }

  pub fn elapsed(&self, pool: &RuntimeObjectPool, handle: AnimationHandle) -> Option<Duration> {
    Some(pool.animations.get(handle.id())?.elapsed)
  }

  pub fn progress(&self, pool: &RuntimeObjectPool, handle: AnimationHandle) -> Option<f64> {
    let playback = pool.animations.get(handle.id())?;
    Some(
      (playback.elapsed.as_secs_f64() / playback.source.duration().as_secs_f64()).clamp(0.0, 1.0),
    )
  }

  pub fn completed_cycles(&self, pool: &RuntimeObjectPool, handle: AnimationHandle) -> Option<u32> {
    Some(pool.animations.get(handle.id())?.completed_cycles)
  }

  pub fn set_speed(
    &self,
    pool: &mut RuntimeObjectPool,
    handle: AnimationHandle,
    speed: f64,
  ) -> bool {
    if !speed.is_finite() || speed <= 0.0 {
      return false;
    }
    let Some(playback) = pool.animations.get_mut(handle.id()) else {
      return false;
    };
    playback.speed = speed;
    true
  }

  pub fn take_events(
    &self,
    pool: &mut RuntimeObjectPool,
    handle: AnimationHandle,
  ) -> Vec<AnimationEvent> {
    let id = handle.id();
    let mut events = Vec::new();
    pool.animations.events.retain(|event| {
      if event.id == id {
        events.push(event.clone());
        false
      } else {
        true
      }
    });
    events
  }

  pub fn take_callback_requests(
    &self,
    pool: &mut RuntimeObjectPool,
  ) -> Vec<AnimationCallbackRequest> {
    pool.animations.callback_requests.drain(..).collect()
  }

  pub fn create_value(
    &self,
    pool: &mut RuntimeObjectPool,
    value: AnimationValue,
  ) -> AnimationValueId {
    pool.animation_values.insert(value)
  }

  pub fn remove_value(&self, pool: &mut RuntimeObjectPool, id: AnimationValueId) -> bool {
    let removed = pool.animation_values.remove(id).is_some();
    if removed {
      pool.remove_animations_targeting(AnimationTarget::Value(id));
    }
    removed
  }

  pub fn value<'a>(
    &self,
    pool: &'a RuntimeObjectPool,
    id: AnimationValueId,
  ) -> Option<&'a AnimationValue> {
    Some(pool.animation_values.get(id)?.resolved())
  }

  pub fn set_value(
    &self,
    pool: &mut RuntimeObjectPool,
    id: AnimationValueId,
    value: AnimationValue,
  ) -> Result<(), AnimationError> {
    let property = pool
      .animation_values
      .get_mut(id)
      .ok_or(AnimationError::StaleValue)?;
    if property.base.kind() != value.kind() {
      return Err(AnimationError::ValueTypeMismatch {
        expected: property.base.kind(),
        actual: value.kind(),
      });
    }
    property.base = value;
    Ok(())
  }

  fn consume_delay(&self, playback: &mut AnimationPlayback, dt: Duration) -> Duration {
    let delay = playback_options_delay(playback);
    if playback.delay_elapsed >= delay {
      return dt;
    }
    let needed = delay.saturating_sub(playback.delay_elapsed);
    let consumed = dt.min(needed);
    playback.delay_elapsed = playback.delay_elapsed.saturating_add(consumed);
    dt.saturating_sub(consumed)
  }

  fn advance(
    &self,
    pool: &mut RuntimeObjectPool,
    id: AnimationId,
    playback: &mut AnimationPlayback,
    mut remaining: Duration,
    output: &mut AnimationUpdate,
  ) {
    let duration = playback.source.duration();
    while !remaining.is_zero() && playback.state == PlaybackState::Playing {
      let segment = duration.saturating_sub(playback.elapsed);
      let step = remaining.min(segment);
      let previous = sample_time(playback, playback.elapsed);
      playback.elapsed = playback.elapsed.saturating_add(step);
      let current = sample_time(playback, playback.elapsed);
      self.emit_markers(pool, id, playback, previous, current, &mut output.events);
      self.sample_playback(id, playback, &mut output.writes);
      remaining = remaining.saturating_sub(step);

      if playback.elapsed < duration {
        break;
      }
      playback.completed_cycles = playback.completed_cycles.saturating_add(1);
      if repeat_finished(playback) {
        self.complete(pool, id, playback, output);
        break;
      }

      self.push_event(
        pool,
        id,
        playback,
        AnimationEventKind::Loop {
          completed: playback.completed_cycles,
        },
        Some(&mut output.events),
      );
      playback.elapsed = Duration::ZERO;
      if playback.repeat.mode == AnimationRepeatMode::PingPong {
        playback.direction = match playback.direction {
          PlaybackDirection::Forward => PlaybackDirection::Reverse,
          PlaybackDirection::Reverse => PlaybackDirection::Forward,
        };
      }
      self.sample_playback(id, playback, &mut output.writes);
    }
  }

  fn complete(
    &self,
    pool: &mut RuntimeObjectPool,
    id: AnimationId,
    playback: &mut AnimationPlayback,
    output: &mut AnimationUpdate,
  ) {
    match playback.end_mode {
      AnimationEndMode::Commit => {
        for (binding, value) in playback.bindings.iter().zip(&playback.current_values) {
          push_commit_and_clear(id, binding, value.clone(), &mut output.writes);
        }
      }
      AnimationEndMode::Restore => {
        for binding in &playback.bindings {
          output.writes.push(AnimationWrite {
            animation_id: id,
            target: binding.target,
            property: binding.property,
            value: None,
            operation: AnimationWriteOperation::ClearOverride,
          });
        }
      }
    }
    playback.state = PlaybackState::Finished;
    self.push_event(
      pool,
      id,
      playback,
      AnimationEventKind::Finished,
      Some(&mut output.events),
    );
  }

  fn sample_playback(
    &self,
    id: AnimationId,
    playback: &mut AnimationPlayback,
    writes: &mut Vec<AnimationWrite>,
  ) {
    let time = sample_time(playback, playback.elapsed);
    for (index, binding) in playback.bindings.iter().enumerate() {
      let Ok(value) = sample_source(&playback.source, binding.track, time) else {
        continue;
      };
      playback.current_values[index] = value.clone();
      writes.push(AnimationWrite {
        animation_id: id,
        target: binding.target,
        property: binding.property,
        value: Some(value),
        operation: AnimationWriteOperation::Override,
      });
    }
  }

  fn emit_markers(
    &self,
    pool: &mut RuntimeObjectPool,
    id: AnimationId,
    playback: &AnimationPlayback,
    previous: Duration,
    current: Duration,
    output: &mut Vec<AnimationEvent>,
  ) {
    let AnimationSource::Clip(clip) = &playback.source else {
      return;
    };
    let marker_names = match playback.direction {
      PlaybackDirection::Forward => clip
        .markers
        .iter()
        .filter(|marker| marker.at > previous && marker.at <= current)
        .map(|marker| marker.name.clone())
        .collect::<Vec<_>>(),
      PlaybackDirection::Reverse => clip
        .markers
        .iter()
        .rev()
        .filter(|marker| marker.at < previous && marker.at >= current)
        .map(|marker| marker.name.clone())
        .collect::<Vec<_>>(),
    };
    for name in marker_names {
      self.push_event(
        pool,
        id,
        playback,
        AnimationEventKind::Marker { name },
        Some(output),
      );
    }
  }

  fn push_event(
    &self,
    pool: &mut RuntimeObjectPool,
    id: AnimationId,
    playback: &AnimationPlayback,
    kind: AnimationEventKind,
    output: Option<&mut Vec<AnimationEvent>>,
  ) {
    let event = AnimationEvent { id, kind };
    if playback.emit_events {
      pool.animations.events.push_back(event.clone());
      if let Some(output) = output {
        output.push(event.clone());
      }
    }
    if let Some(callback) = playback.callback {
      pool
        .animations
        .callback_requests
        .push_back(AnimationCallbackRequest { callback, event });
    }
  }

  fn apply_internal_writes(&self, pool: &mut RuntimeObjectPool, writes: &[AnimationWrite]) {
    for write in writes {
      match write.target {
        AnimationTarget::Value(id) if write.property == AnimationProperty::Value => {
          if let Some(value) = pool.animation_values.get_mut(id) {
            apply_to_animated_value(value, write);
          }
        }
        AnimationTarget::Effect(id) => {
          let parameter = match write.property {
            AnimationProperty::EffectPhase => EffectParameterId::PHASE,
            AnimationProperty::EffectStrength => EffectParameterId::STRENGTH,
            AnimationProperty::EffectParameter(parameter) => parameter,
            _ => continue,
          };
          if let Some(value) = pool
            .character_effects
            .get_mut(id)
            .and_then(|effect| effect.parameters.get_mut(&parameter))
          {
            apply_to_animated_value(value, write);
          }
        }
        _ => {}
      }
    }
  }

  fn validate(
    &self,
    source: &AnimationSource,
    bindings: &[AnimationBinding],
    options: &super::AnimationPlaybackOptions,
  ) -> Result<(), AnimationError> {
    if source.duration().is_zero() {
      return Err(AnimationError::InvalidDuration);
    }
    if !options.speed.is_finite() || options.speed <= 0.0 {
      return Err(AnimationError::InvalidSpeed);
    }
    if options.repeat.count == AnimationRepeatCount::Finite(0) {
      return Err(AnimationError::InvalidRepeatCount);
    }
    validate_source(source)?;
    if bindings.is_empty() {
      return Err(AnimationError::InvalidBinding(0));
    }
    for (index, binding) in bindings.iter().enumerate() {
      if binding.track >= source.track_count() {
        return Err(AnimationError::InvalidTrack(binding.track));
      }
      if let Some(property) = source.track_property(binding.track)
        && property != binding.property
        && !matches!(binding.target, AnimationTarget::Value(_))
      {
        return Err(AnimationError::InvalidBinding(index));
      }
      let sampled = sample_source(source, binding.track, Duration::ZERO)?;
      if binding.initial_value.kind() != sampled.kind() {
        return Err(AnimationError::ValueTypeMismatch {
          expected: sampled.kind(),
          actual: binding.initial_value.kind(),
        });
      }
      validate_property(binding.target, binding.property, sampled.kind())?;
    }
    Ok(())
  }

  fn validate_internal_targets(
    &self,
    pool: &RuntimeObjectPool,
    bindings: &[AnimationBinding],
  ) -> Result<(), AnimationError> {
    for binding in bindings {
      match binding.target {
        AnimationTarget::Value(id) => {
          let value = pool
            .animation_values
            .get(id)
            .ok_or(AnimationError::StaleValue)?;
          if value.base.kind() != binding.initial_value.kind() {
            return Err(AnimationError::ValueTypeMismatch {
              expected: value.base.kind(),
              actual: binding.initial_value.kind(),
            });
          }
        }
        AnimationTarget::Effect(id) => {
          let effect = pool
            .character_effects
            .get(id)
            .ok_or(AnimationError::StaleEffect)?;
          let parameter = match binding.property {
            AnimationProperty::EffectPhase => EffectParameterId::PHASE,
            AnimationProperty::EffectStrength => EffectParameterId::STRENGTH,
            AnimationProperty::EffectParameter(parameter) => parameter,
            _ => continue,
          };
          let value = effect
            .parameters
            .get(&parameter)
            .ok_or(AnimationError::MissingEffectParameter(parameter))?;
          if value.base.kind() != binding.initial_value.kind() {
            return Err(AnimationError::ValueTypeMismatch {
              expected: value.base.kind(),
              actual: binding.initial_value.kind(),
            });
          }
        }
        _ => {}
      }
    }
    Ok(())
  }
}

fn playback_options_delay(playback: &AnimationPlayback) -> Duration {
  playback.delay
}

fn reset_playback_time(playback: &mut AnimationPlayback) {
  playback.elapsed = Duration::ZERO;
  playback.delay_elapsed = Duration::ZERO;
  playback.completed_cycles = 0;
  playback.direction = PlaybackDirection::Forward;
}

fn scale_duration(duration: Duration, speed: f64) -> Duration {
  Duration::from_secs_f64((duration.as_secs_f64() * speed).max(0.0))
}

fn repeat_finished(playback: &AnimationPlayback) -> bool {
  matches!(
    playback.repeat.count,
    AnimationRepeatCount::Finite(count) if playback.completed_cycles >= count
  )
}

fn sample_time(playback: &AnimationPlayback, elapsed: Duration) -> Duration {
  match playback.direction {
    PlaybackDirection::Forward => elapsed,
    PlaybackDirection::Reverse => playback.source.duration().saturating_sub(elapsed),
  }
}

fn sample_source(
  source: &AnimationSource,
  track_index: usize,
  time: Duration,
) -> Result<AnimationValue, AnimationError> {
  match source {
    AnimationSource::Tween(definition) => {
      if track_index != 0 {
        return Err(AnimationError::InvalidTrack(track_index));
      }
      let progress = (time.as_secs_f64() / definition.duration.as_secs_f64()).clamp(0.0, 1.0);
      let sampled = sample_progress(progress, definition.easing, definition.interpolation);
      definition
        .from
        .interpolate(&definition.to, sampled, definition.interpolation)
    }
    AnimationSource::Clip(clip) => {
      let track = clip
        .tracks
        .get(track_index)
        .ok_or(AnimationError::InvalidTrack(track_index))?;
      let first = track
        .keyframes
        .first()
        .ok_or(AnimationError::InvalidKeyframes(track_index))?;
      let Some(right_index) = track
        .keyframes
        .iter()
        .position(|keyframe| keyframe.at > time)
      else {
        return Ok(
          track
            .keyframes
            .last()
            .expect("track is non-empty")
            .value
            .clone(),
        );
      };
      if right_index == 0 {
        return Ok(first.value.clone());
      }
      let left = &track.keyframes[right_index - 1];
      let right = &track.keyframes[right_index];
      let span = right.at.saturating_sub(left.at);
      let progress = if span.is_zero() {
        1.0
      } else {
        time.saturating_sub(left.at).as_secs_f64() / span.as_secs_f64()
      };
      let sampled = sample_progress(progress, right.easing, right.interpolation);
      left
        .value
        .interpolate(&right.value, sampled, right.interpolation)
    }
  }
}

fn sample_progress(
  progress: f64,
  easing: super::AnimationEasing,
  interpolation: AnimationInterpolation,
) -> f64 {
  if interpolation == AnimationInterpolation::Step {
    progress.clamp(0.0, 1.0)
  } else {
    easing::sample(easing, progress)
  }
}

fn validate_source(source: &AnimationSource) -> Result<(), AnimationError> {
  match source {
    AnimationSource::Tween(definition) => {
      if definition.duration.is_zero() {
        return Err(AnimationError::InvalidDuration);
      }
      validate_interpolation(
        definition.from.kind(),
        definition.to.kind(),
        definition.interpolation,
      )
    }
    AnimationSource::Clip(clip) => {
      if clip.duration.is_zero() || clip.tracks.is_empty() {
        return Err(AnimationError::InvalidDuration);
      }
      for (track_index, track) in clip.tracks.iter().enumerate() {
        if track.keyframes.is_empty()
          || track
            .keyframes
            .windows(2)
            .any(|pair| pair[0].at >= pair[1].at)
          || track
            .keyframes
            .last()
            .is_some_and(|keyframe| keyframe.at > clip.duration)
        {
          return Err(AnimationError::InvalidKeyframes(track_index));
        }
        for pair in track.keyframes.windows(2) {
          validate_interpolation(
            pair[0].value.kind(),
            pair[1].value.kind(),
            pair[1].interpolation,
          )?;
        }
      }
      if clip.markers.iter().any(|marker| marker.at > clip.duration) {
        return Err(AnimationError::InvalidDuration);
      }
      Ok(())
    }
  }
}

fn validate_interpolation(
  from: AnimationValueKind,
  to: AnimationValueKind,
  interpolation: AnimationInterpolation,
) -> Result<(), AnimationError> {
  if from != to {
    return Err(AnimationError::ValueTypeMismatch {
      expected: from,
      actual: to,
    });
  }
  if matches!(
    from,
    AnimationValueKind::Bool | AnimationValueKind::Text | AnimationValueKind::CharacterFrame
  ) && interpolation != AnimationInterpolation::Step
  {
    return Err(AnimationError::DiscreteInterpolationRequired(from));
  }
  Ok(())
}

fn validate_property(
  target: AnimationTarget,
  property: AnimationProperty,
  kind: AnimationValueKind,
) -> Result<(), AnimationError> {
  let supported = match target {
    AnimationTarget::Value(_) => property == AnimationProperty::Value,
    AnimationTarget::Effect(_) => matches!(
      property,
      AnimationProperty::EffectPhase
        | AnimationProperty::EffectStrength
        | AnimationProperty::EffectParameter(_)
    ),
    AnimationTarget::Game(_) => !matches!(
      property,
      AnimationProperty::Value | AnimationProperty::EffectParameter(_)
    ),
    AnimationTarget::Ui(reference) => match reference.kind {
      super::UiObjectKind::Slice => matches!(
        property,
        AnimationProperty::OffsetX
          | AnimationProperty::OffsetY
          | AnimationProperty::WidthOffset
          | AnimationProperty::HeightOffset
          | AnimationProperty::Visible
      ),
      super::UiObjectKind::ScrollBox => matches!(
        property,
        AnimationProperty::OffsetX
          | AnimationProperty::OffsetY
          | AnimationProperty::WidthOffset
          | AnimationProperty::HeightOffset
          | AnimationProperty::Visible
          | AnimationProperty::ScrollX
          | AnimationProperty::ScrollY
      ),
      super::UiObjectKind::ProgressBar => matches!(
        property,
        AnimationProperty::OffsetX
          | AnimationProperty::OffsetY
          | AnimationProperty::Progress
          | AnimationProperty::Foreground
          | AnimationProperty::Background
          | AnimationProperty::Visible
      ),
      _ => !matches!(
        property,
        AnimationProperty::Progress
          | AnimationProperty::ScrollX
          | AnimationProperty::ScrollY
          | AnimationProperty::EffectParameter(_)
          | AnimationProperty::Value
      ),
    },
  };
  if !supported {
    return Err(AnimationError::PropertyNotSupported { target, property });
  }
  let type_matches = match property {
    AnimationProperty::OffsetX
    | AnimationProperty::OffsetY
    | AnimationProperty::WidthOffset
    | AnimationProperty::HeightOffset
    | AnimationProperty::Progress
    | AnimationProperty::ScrollX
    | AnimationProperty::ScrollY
    | AnimationProperty::EffectPhase
    | AnimationProperty::EffectStrength
    | AnimationProperty::EffectParameter(_) => matches!(
      kind,
      AnimationValueKind::Float | AnimationValueKind::Integer | AnimationValueKind::Unsigned
    ),
    AnimationProperty::Foreground
    | AnimationProperty::Background
    | AnimationProperty::BorderForeground => kind == AnimationValueKind::Color,
    AnimationProperty::Visible => kind == AnimationValueKind::Bool,
    AnimationProperty::VisibleGraphemes | AnimationProperty::VisibleLines => {
      kind == AnimationValueKind::Unsigned
    }
    AnimationProperty::GlyphFrame => matches!(
      kind,
      AnimationValueKind::CharacterFrame | AnimationValueKind::Text | AnimationValueKind::Unsigned
    ),
    AnimationProperty::Value => true,
  };
  if !type_matches {
    return Err(AnimationError::PropertyTypeMismatch {
      property,
      actual: kind,
    });
  }
  Ok(())
}

fn apply_to_animated_value(value: &mut AnimatedValue, write: &AnimationWrite) {
  match write.operation {
    AnimationWriteOperation::Override => value.animation_override = write.value.clone(),
    AnimationWriteOperation::Commit => {
      if let Some(new_value) = &write.value {
        value.base = new_value.clone();
      }
    }
    AnimationWriteOperation::ClearOverride => value.animation_override = None,
  }
}

fn push_commit_and_clear(
  id: AnimationId,
  binding: &AnimationBinding,
  value: AnimationValue,
  writes: &mut Vec<AnimationWrite>,
) {
  writes.push(AnimationWrite {
    animation_id: id,
    target: binding.target,
    property: binding.property,
    value: Some(value),
    operation: AnimationWriteOperation::Commit,
  });
  writes.push(AnimationWrite {
    animation_id: id,
    target: binding.target,
    property: binding.property,
    value: None,
    operation: AnimationWriteOperation::ClearOverride,
  });
}
