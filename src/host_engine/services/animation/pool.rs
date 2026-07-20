use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use super::{
  AnimationBinding, AnimationCallbackRequest, AnimationClock, AnimationEndMode, AnimationEvent,
  AnimationId, AnimationOwner, AnimationPlaybackOptions, AnimationRepeatOptions, AnimationSource,
  AnimationValue, AnimationValueId, CellEffectId, EffectParameterId, PlaybackDirection,
  PlaybackState,
};

pub(crate) struct Arena<T> {
  slots: Vec<ArenaSlot<T>>,
}

struct ArenaSlot<T> {
  generation: u32,
  value: Option<T>,
}

impl<T> Arena<T> {
  pub(crate) fn new() -> Self {
    Self { slots: Vec::new() }
  }

  pub(crate) fn insert(&mut self, value: T) -> (u32, u32) {
    if let Some((index, slot)) = self
      .slots
      .iter_mut()
      .enumerate()
      .find(|(_, slot)| slot.value.is_none())
    {
      slot.value = Some(value);
      return (index as u32, slot.generation);
    }
    let index = self.slots.len() as u32;
    self.slots.push(ArenaSlot {
      generation: 1,
      value: Some(value),
    });
    (index, 1)
  }

  pub(crate) fn get(&self, index: u32, generation: u32) -> Option<&T> {
    let slot = self.slots.get(index as usize)?;
    (slot.generation == generation).then_some(slot.value.as_ref()?)
  }

  pub(crate) fn get_mut(&mut self, index: u32, generation: u32) -> Option<&mut T> {
    let slot = self.slots.get_mut(index as usize)?;
    (slot.generation == generation).then_some(slot.value.as_mut()?)
  }

  pub(crate) fn remove(&mut self, index: u32, generation: u32) -> Option<T> {
    let slot = self.slots.get_mut(index as usize)?;
    if slot.generation != generation {
      return None;
    }
    let value = slot.value.take()?;
    slot.generation = slot.generation.wrapping_add(1).max(1);
    Some(value)
  }

  pub(crate) fn keys(&self) -> Vec<(u32, u32)> {
    self
      .slots
      .iter()
      .enumerate()
      .filter_map(|(index, slot)| slot.value.as_ref().map(|_| (index as u32, slot.generation)))
      .collect()
  }
}

#[derive(Clone)]
pub(crate) struct AnimationPlayback {
  pub(crate) owner: AnimationOwner,
  pub(crate) source: AnimationSource,
  pub(crate) bindings: Vec<AnimationBinding>,
  pub(crate) elapsed: Duration,
  pub(crate) delay: Duration,
  pub(crate) delay_elapsed: Duration,
  pub(crate) speed: f64,
  pub(crate) state: PlaybackState,
  pub(crate) clock: AnimationClock,
  pub(crate) repeat: AnimationRepeatOptions,
  pub(crate) end_mode: AnimationEndMode,
  pub(crate) direction: PlaybackDirection,
  pub(crate) completed_cycles: u32,
  pub(crate) emit_events: bool,
  pub(crate) callback: Option<super::AnimationCallbackId>,
  pub(crate) current_values: Vec<AnimationValue>,
}

impl AnimationPlayback {
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

pub(crate) struct AnimationPool {
  playbacks: Arena<AnimationPlayback>,
  pub(crate) events: VecDeque<AnimationEvent>,
  pub(crate) callback_requests: VecDeque<AnimationCallbackRequest>,
}

impl AnimationPool {
  pub(crate) fn new() -> Self {
    Self {
      playbacks: Arena::new(),
      events: VecDeque::new(),
      callback_requests: VecDeque::new(),
    }
  }

  pub(crate) fn insert(&mut self, playback: AnimationPlayback) -> AnimationId {
    let (index, generation) = self.playbacks.insert(playback);
    AnimationId::new(index, generation)
  }

  pub(crate) fn get(&self, id: AnimationId) -> Option<&AnimationPlayback> {
    self.playbacks.get(id.index(), id.generation())
  }

  pub(crate) fn get_mut(&mut self, id: AnimationId) -> Option<&mut AnimationPlayback> {
    self.playbacks.get_mut(id.index(), id.generation())
  }

  pub(crate) fn remove(&mut self, id: AnimationId) -> Option<AnimationPlayback> {
    self.playbacks.remove(id.index(), id.generation())
  }

  pub(crate) fn ids(&self) -> Vec<AnimationId> {
    self
      .playbacks
      .keys()
      .into_iter()
      .map(|(index, generation)| AnimationId::new(index, generation))
      .collect()
  }

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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AnimatedValue {
  pub(crate) base: AnimationValue,
  pub(crate) animation_override: Option<AnimationValue>,
}

impl AnimatedValue {
  pub(crate) fn new(base: AnimationValue) -> Self {
    Self {
      base,
      animation_override: None,
    }
  }

  pub(crate) fn resolved(&self) -> &AnimationValue {
    self.animation_override.as_ref().unwrap_or(&self.base)
  }
}

pub(crate) struct AnimationValuePool {
  values: Arena<AnimatedValue>,
}

impl AnimationValuePool {
  pub(crate) fn new() -> Self {
    Self {
      values: Arena::new(),
    }
  }

  pub(crate) fn insert(&mut self, value: AnimationValue) -> AnimationValueId {
    let (index, generation) = self.values.insert(AnimatedValue::new(value));
    AnimationValueId::new(index, generation)
  }

  pub(crate) fn get(&self, id: AnimationValueId) -> Option<&AnimatedValue> {
    self.values.get(id.index(), id.generation())
  }

  pub(crate) fn get_mut(&mut self, id: AnimationValueId) -> Option<&mut AnimatedValue> {
    self.values.get_mut(id.index(), id.generation())
  }

  pub(crate) fn remove(&mut self, id: AnimationValueId) -> Option<AnimatedValue> {
    self.values.remove(id.index(), id.generation())
  }
}

pub(crate) struct CharacterEffect {
  pub(crate) parameters: HashMap<EffectParameterId, AnimatedValue>,
}

pub(crate) struct CharacterEffectPool {
  effects: Arena<CharacterEffect>,
}

impl CharacterEffectPool {
  pub(crate) fn new() -> Self {
    Self {
      effects: Arena::new(),
    }
  }

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

  pub(crate) fn get(&self, id: CellEffectId) -> Option<&CharacterEffect> {
    self.effects.get(id.index(), id.generation())
  }

  pub(crate) fn get_mut(&mut self, id: CellEffectId) -> Option<&mut CharacterEffect> {
    self.effects.get_mut(id.index(), id.generation())
  }

  pub(crate) fn remove(&mut self, id: CellEffectId) -> Option<CharacterEffect> {
    self.effects.remove(id.index(), id.generation())
  }
}
