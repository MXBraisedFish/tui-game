//! Generational object storage and ownership-based lookup and removal.

use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use super::{
  AnimationBinding, AnimationCallbackRequest, AnimationClock, AnimationEndMode, AnimationEvent,
  AnimationId, AnimationOwner, AnimationPlaybackOptions, AnimationRepeatOptions, AnimationSource,
  AnimationTarget, AnimationValue, AnimationValueId, CellEffectId, EffectParameterId,
  PlaybackDirection, PlaybackState,
};

pub(crate) use tg_core_arena::Arena;

/// The animation playback representation used by this module.
///
/// # Fields
///
/// * `owner` - The owner whose resources are being addressed.
/// * `source` - The animation source carried by this animation playback.
/// * `bindings` - The ordered bindings retained by this owner.
/// * `elapsed` - The elapsed represented as a duration.
/// * `delay` - The delay represented as a duration.
/// * `delay_elapsed` - The delay elapsed.
/// * `speed` - The playback or animation speed multiplier.
/// * `state` - The playback state carried by this animation playback.
/// * `clock` - The clock.
/// * `repeat` - The repeat.
/// * `end_mode` - The end mode.
/// * `direction` - The direction.
/// * `completed_cycles` - The completed cycles.
/// * `emit_events` - Whether playback transitions enter the event queue.
/// * `callback` - The callback.
/// * `current_values` - The ordered current values retained by this owner.
#[derive(Clone)]
pub(crate) struct AnimationPlayback {
  /// The owner whose resources are being addressed.
  pub(crate) owner: AnimationOwner,
  /// The animation source carried by this animation playback.
  pub(crate) source: AnimationSource,
  /// The ordered bindings retained by this owner.
  pub(crate) bindings: Vec<AnimationBinding>,
  /// The elapsed represented as a duration.
  pub(crate) elapsed: Duration,
  /// The delay represented as a duration.
  pub(crate) delay: Duration,
  /// The delay elapsed.
  pub(crate) delay_elapsed: Duration,
  /// The playback or animation speed multiplier.
  pub(crate) speed: f64,
  /// The playback state carried by this animation playback.
  pub(crate) state: PlaybackState,
  /// The clock.
  pub(crate) clock: AnimationClock,
  /// The repeat.
  pub(crate) repeat: AnimationRepeatOptions,
  /// The end mode.
  pub(crate) end_mode: AnimationEndMode,
  /// The direction.
  pub(crate) direction: PlaybackDirection,
  /// The completed cycles.
  pub(crate) completed_cycles: u32,
  /// Whether playback transitions enter the event queue.
  pub(crate) emit_events: bool,
  /// The callback.
  pub(crate) callback: Option<super::AnimationCallbackId>,
  /// The ordered current values retained by this owner.
  pub(crate) current_values: Vec<AnimationValue>,
}

impl AnimationPlayback {
  /// Create an animation playback initialized from `owner`, `source`, `bindings`, `options`.
  ///
  /// # Arguments
  ///
  /// * `owner` - The owner whose resources are being addressed.
  /// * `source` - The source value or package origin.
  /// * `bindings` - The bindings.
  /// * `options` - The validated options for the operation.
  pub(crate) fn new(
    owner: AnimationOwner,
    source: AnimationSource,
    bindings: Vec<AnimationBinding>,
    options: AnimationPlaybackOptions,
  ) -> Self {
    let current_values = bindings
      .iter()
      .map(|binding| binding.initial_value.clone())
      .collect();
    Self {
      owner,
      source,
      bindings,
      elapsed: Duration::ZERO,
      delay: options.delay,
      delay_elapsed: Duration::ZERO,
      speed: options.speed,
      state: if options.auto_play {
        PlaybackState::Playing
      } else {
        PlaybackState::Idle
      },
      clock: options.clock,
      repeat: options.repeat,
      end_mode: options.end_mode,
      direction: PlaybackDirection::Forward,
      completed_cycles: 0,
      emit_events: options.emit_events,
      callback: options.callback,
      current_values,
    }
  }
}

/// Owned animation instances with live-identifier lookup and removal.
///
/// # Fields
///
/// * `events` - Pending events retained in delivery order.
/// * `callback_requests` - The ordered callback requests retained by this owner.
pub(crate) struct AnimationPool {
  playbacks: Arena<AnimationPlayback>,
  /// Pending events retained in delivery order.
  pub(crate) events: VecDeque<AnimationEvent>,
  /// The ordered callback requests retained by this owner.
  pub(crate) callback_requests: VecDeque<AnimationCallbackRequest>,
}

impl AnimationPool {
  /// Create an animation pool with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      playbacks: Arena::new(),
      events: VecDeque::new(),
      callback_requests: VecDeque::new(),
    }
  }

  /// Store a value in the animation pool and return its new identity.
  pub(crate) fn insert(&mut self, playback: AnimationPlayback) -> AnimationId {
    let (index, generation) = self.playbacks.insert(playback);
    AnimationId::new(index, generation)
  }

  /// Return access to the value only when its slot and generation are still live.
  pub(crate) fn get(&self, id: AnimationId) -> Option<&AnimationPlayback> {
    self.playbacks.get(id.index(), id.generation())
  }

  /// Return mutable access to the value only when its slot and generation are still live.
  pub(crate) fn get_mut(&mut self, id: AnimationId) -> Option<&mut AnimationPlayback> {
    self.playbacks.get_mut(id.index(), id.generation())
  }

  /// Remove the identified animation object and release its owned state.
  pub(crate) fn remove(&mut self, id: AnimationId) -> Option<AnimationPlayback> {
    self.playbacks.remove(id.index(), id.generation())
  }

  /// Return the ids for the addressed object.
  pub(crate) fn ids(&self) -> Vec<AnimationId> {
    self
      .playbacks
      .keys()
      .into_iter()
      .map(|(index, generation)| AnimationId::new(index, generation))
      .collect()
  }

  /// Collect the live animation identifiers associated with the requested owner.
  pub(crate) fn ids_owned_by(&self, owner: AnimationOwner) -> Vec<AnimationId> {
    self
      .ids()
      .into_iter()
      .filter(|id| {
        self
          .get(*id)
          .is_some_and(|playback| playback.owner == owner)
      })
      .collect()
  }
}

/// The animated value representation used by this module.
///
/// # Fields
///
/// * `base` - The coordinate or path base used for relative resolution.
/// * `animation_override` - The animation override.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AnimatedValue {
  /// The coordinate or path base used for relative resolution.
  pub(crate) base: AnimationValue,
  /// The animation override.
  pub(crate) animation_override: Option<AnimationValue>,
}

impl AnimatedValue {
  /// Create an animated value initialized from `base`.
  pub(crate) fn new(base: AnimationValue) -> Self {
    Self {
      base,
      animation_override: None,
    }
  }

  /// Return the current resolved.
  pub(crate) fn resolved(&self) -> &AnimationValue {
    self.animation_override.as_ref().unwrap_or(&self.base)
  }
}

/// Owned animation value instances with live-identifier lookup and removal.
pub(crate) struct AnimationValuePool {
  values: Arena<AnimatedValue>,
}

impl AnimationValuePool {
  /// Create an animation value pool with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      values: Arena::new(),
    }
  }

  /// Store a value in the animation value pool and return its new identity.
  pub(crate) fn insert(&mut self, value: AnimationValue) -> AnimationValueId {
    let (index, generation) = self.values.insert(AnimatedValue::new(value));
    AnimationValueId::new(index, generation)
  }

  /// Return access to the value only when its slot and generation are still live.
  pub(crate) fn get(&self, id: AnimationValueId) -> Option<&AnimatedValue> {
    self.values.get(id.index(), id.generation())
  }

  /// Return mutable access to the value only when its slot and generation are still live.
  pub(crate) fn get_mut(&mut self, id: AnimationValueId) -> Option<&mut AnimatedValue> {
    self.values.get_mut(id.index(), id.generation())
  }

  /// Remove the identified animation object and release its owned state.
  pub(crate) fn remove(&mut self, id: AnimationValueId) -> Option<AnimatedValue> {
    self.values.remove(id.index(), id.generation())
  }
}

/// The character effect representation used by this module.
///
/// # Fields
///
/// * `parameters` - The parameters indexed by their declared keys.
pub(crate) struct CharacterEffect {
  /// The parameters indexed by their declared keys.
  pub(crate) parameters: HashMap<EffectParameterId, AnimatedValue>,
}

/// Owned character effect instances with live-identifier lookup and removal.
pub(crate) struct CharacterEffectPool {
  effects: Arena<CharacterEffect>,
}

impl CharacterEffectPool {
  /// Create a character effect pool with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      effects: Arena::new(),
    }
  }

  /// Store a value in the character effect pool and return its new identity.
  pub(crate) fn insert(
    &mut self,
    parameters: HashMap<EffectParameterId, AnimationValue>,
  ) -> CellEffectId {
    let parameters = parameters
      .into_iter()
      .map(|(id, value)| (id, AnimatedValue::new(value)))
      .collect();
    let (index, generation) = self.effects.insert(CharacterEffect { parameters });
    CellEffectId::new(index, generation)
  }

  /// Return access to the value only when its slot and generation are still live.
  pub(crate) fn get(&self, id: CellEffectId) -> Option<&CharacterEffect> {
    self.effects.get(id.index(), id.generation())
  }

  /// Return mutable access to the value only when its slot and generation are still live.
  pub(crate) fn get_mut(&mut self, id: CellEffectId) -> Option<&mut CharacterEffect> {
    self.effects.get_mut(id.index(), id.generation())
  }

  /// Remove the identified animation object and release its owned state.
  pub(crate) fn remove(&mut self, id: CellEffectId) -> Option<CharacterEffect> {
    self.effects.remove(id.index(), id.generation())
  }
}

/// The collection of owned animation instances and their queued events.
///
/// # Fields
///
/// * `animations` - The animations.
/// * `animation_values` - The animation values.
/// * `character_effects` - The character effects.
pub struct AnimationObjects {
  /// The animations.
  pub(crate) animations: AnimationPool,
  /// The animation values.
  pub(crate) animation_values: AnimationValuePool,
  /// The character effects.
  pub(crate) character_effects: CharacterEffectPool,
}

impl AnimationObjects {
  /// Create an animation objects with its initial state.
  pub fn new() -> Self {
    Self {
      animations: AnimationPool::new(),
      animation_values: AnimationValuePool::new(),
      character_effects: CharacterEffectPool::new(),
    }
  }

  /// Return the current animation count.
  pub fn animation_count(&self) -> usize {
    self.animations.ids().len()
  }

  /// Remove playbacks whose bindings address the target being released.
  pub(crate) fn remove_animations_targeting(&mut self, target: AnimationTarget) {
    let ids = self
      .animations
      .ids()
      .into_iter()
      .filter(|id| {
        self.animations.get(*id).is_some_and(|playback| {
          playback.owner == AnimationOwner::Object(target)
            || playback
              .bindings
              .iter()
              .any(|binding| binding.target == target)
        })
      })
      .collect::<Vec<_>>();
    for id in ids {
      self.animations.remove(id);
      self.animations.events.retain(|event| event.id != id);
      self
        .animations
        .callback_requests
        .retain(|request| request.event.id != id);
    }
  }
}

impl Default for AnimationObjects {
  fn default() -> Self {
    Self::new()
  }
}
