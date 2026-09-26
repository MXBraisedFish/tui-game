use std::collections::HashMap;

use super::AnimationObjects;

use super::{AnimationError, AnimationTarget, AnimationValue, CellEffectId, EffectParameterId};

/// The host-side entry point for managing character effect parameters. The per-cell effect
/// algorithm itself is left to the render objects that interpret these parameters.
pub struct CharacterEffectService;

impl Default for CharacterEffectService {
  fn default() -> Self {
    Self::new()
  }
}

impl CharacterEffectService {
  pub fn new() -> Self {
    Self
  }

  pub fn create(
    &self,
    pool: &mut AnimationObjects,
    parameters: HashMap<EffectParameterId, AnimationValue>,
  ) -> CellEffectId {
    pool.character_effects.insert(parameters)
  }

  pub fn remove(&self, pool: &mut AnimationObjects, id: CellEffectId) -> bool {
    let removed = pool.character_effects.remove(id).is_some();
    if removed {
      pool.remove_animations_targeting(AnimationTarget::Effect(id));
    }
    removed
  }

  pub fn exists(&self, pool: &AnimationObjects, id: CellEffectId) -> bool {
    pool.character_effects.get(id).is_some()
  }

  pub fn parameter<'a>(
    &self,
    pool: &'a AnimationObjects,
    id: CellEffectId,
    parameter: EffectParameterId,
  ) -> Option<&'a AnimationValue> {
    Some(
      pool
        .character_effects
        .get(id)?
        .parameters
        .get(&parameter)?
        .resolved(),
    )
  }

  pub fn set_parameter(
    &self,
    pool: &mut AnimationObjects,
    id: CellEffectId,
    parameter: EffectParameterId,
    value: AnimationValue,
  ) -> Result<(), AnimationError> {
    let effect = pool
      .character_effects
      .get_mut(id)
      .ok_or(AnimationError::StaleEffect)?;
    let property = effect
      .parameters
      .get_mut(&parameter)
      .ok_or(AnimationError::MissingEffectParameter(parameter))?;
    if property.base.kind() != value.kind() {
      return Err(AnimationError::ValueTypeMismatch {
        expected: property.base.kind(),
        actual: value.kind(),
      });
    }
    property.base = value;
    Ok(())
  }

  pub fn clear_override(
    &self,
    pool: &mut AnimationObjects,
    id: CellEffectId,
    parameter: EffectParameterId,
  ) -> bool {
    let Some(property) = pool
      .character_effects
      .get_mut(id)
      .and_then(|effect| effect.parameters.get_mut(&parameter))
    else {
      return false;
    };
    property.animation_override = None;
    true
  }
}
