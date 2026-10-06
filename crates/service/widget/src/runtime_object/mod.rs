//! Non-UI objects owned by a widget runtime pool.

use tg_service_animation::AnimationObjects;
use tg_service_random::RandomGeneratorObjects;
use tg_service_time::TimeObjects;

/// Non-UI runtime objects owned by one view or session.
///
/// # Fields
///
/// * `time` - Whether the printed log header includes time.
/// * `random_generators` - The random generators.
/// * `animation` - The animation.
pub struct RuntimeObjectPool {
  /// Whether the printed log header includes time.
  pub time: TimeObjects,
  /// The random generators.
  pub random_generators: RandomGeneratorObjects,
  /// The animation.
  pub animation: AnimationObjects,
}

impl RuntimeObjectPool {
  /// Create a runtime object pool with its initial state.
  pub fn new() -> Self {
    Self {
      time: TimeObjects::new(),
      random_generators: RandomGeneratorObjects::new(),
      animation: AnimationObjects::new(),
    }
  }
}

impl Default for RuntimeObjectPool {
  fn default() -> Self {
    Self::new()
  }
}

/// The contract for accessing or implementing runtime object pool owner.
pub trait RuntimeObjectPoolOwner {
  /// Return the current runtime objects.
  fn runtime_objects(&self) -> &RuntimeObjectPool;
  /// Return mutable access to the owned runtime objects.
  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool;
}
