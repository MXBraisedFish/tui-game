//! Cell-effect instances and explicitly overridden effect parameters.

use std::collections::HashMap;

use super::AnimationObjects;

use super::{AnimationError, AnimationTarget, AnimationValue, CellEffectId, EffectParameterId};

/// The public entry point for character effect operations.
pub struct CharacterEffectService;

impl Default for CharacterEffectService {
  fn default() -> Self {
    Self::new()
  }
}

impl CharacterEffectService {
  /// Create a character effect service with its initial state.
  pub fn new() -> Self {
    Self
  }

  /// Create an owned character effect object and return its identity.
  pub fn create(
    &self,
    pool: &mut AnimationObjects,
    parameters: HashMap<EffectParameterId, AnimationValue>,
  ) -> CellEffectId {
    pool.character_effects.insert(parameters)
  }

  /// Remove the identified animation object and release its owned state.
  pub fn remove(&self, pool: &mut AnimationObjects, id: CellEffectId) -> bool {
    let removed = pool.character_effects.remove(id).is_some();
    if removed {
      pool.remove_animations_targeting(AnimationTarget::Effect(id));
    }
    removed
  }

  /// Report whether the identified animation object is still present.
  pub fn exists(&self, pool: &AnimationObjects, id: CellEffectId) -> bool {
    pool.character_effects.get(id).is_some()
  }

  /// Return the resolved effect parameter, preferring its animation override when present.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `parameter` - The parameter.
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

  /// Update the parameter used by this character effect service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `parameter` - The parameter.
  /// * `value` - The value to store or convert.
  ///
  /// # Errors
  ///
  /// Return `StaleEffect` for a removed effect, `MissingEffectParameter` for an absent parameter,
  /// or `ValueTypeMismatch` for an incompatible value.
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

  /// Clear the override retained by this character effect service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `parameter` - The parameter.
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
