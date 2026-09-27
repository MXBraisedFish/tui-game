use rand::{Rng, SeedableRng};

use super::objects::{
  RandomConfiguration, RandomConfiguredRange, RandomGeneratedValue, RandomGenerator,
  RandomGeneratorId, RandomGeneratorObjects, RandomSeed, RandomSnapshot,
};

pub struct RandomService;

impl RandomService {
  pub fn new() -> Self {
    Self
  }

  pub fn create(&self, generators: &mut RandomGeneratorObjects, seed: RandomSeed) -> RandomGeneratorId {
    generators.create(RandomGenerator::new(seed))
  }

  pub fn create_configured(
    &self,
    generators: &mut RandomGeneratorObjects,
    configuration: RandomConfiguration,
  ) -> RandomGeneratorId {
    let mut generator = RandomGenerator::new(RandomSeed::U64(configuration.seed as u64));
    generator.configuration = Some(configuration);
    generators.create(generator)
  }

  pub fn configuration(
    &self,
    generators: &RandomGeneratorObjects,
    id: RandomGeneratorId,
  ) -> Option<RandomConfiguration> {
    generators.generators.get(&id)?.configuration
  }

  pub fn set_configuration(
    &self,
    generators: &mut RandomGeneratorObjects,
    id: RandomGeneratorId,
    configuration: RandomConfiguration,
  ) -> bool {
    let Some(generator) = generators.generators.get_mut(&id) else {
      return false;
    };
    if generator
      .configuration
      .is_some_and(|current| current.seed != configuration.seed)
    {
      generator.reseed(RandomSeed::U64(configuration.seed as u64));
    }
    generator.configuration = Some(configuration);
    true
  }

  pub fn configured_ids(&self, generators: &RandomGeneratorObjects) -> Vec<RandomGeneratorId> {
    let mut ids = generators
      .generators
      .iter()
      .filter_map(|(id, generator)| generator.configuration.map(|_| *id))
      .collect::<Vec<_>>();
    ids.sort_by_key(|id| id.0);
    ids
  }

  pub fn clear_configured(&self, generators: &mut RandomGeneratorObjects) {
    generators
      .generators
      .retain(|_, generator| generator.configuration.is_none());
  }

  pub fn generate_configured(
    &self,
    generators: &mut RandomGeneratorObjects,
    id: RandomGeneratorId,
  ) -> Option<RandomGeneratedValue> {
    let generator = generators.generators.get_mut(&id)?;
    let configuration = generator.configuration?;
    let mut rng = rand_chacha::ChaCha8Rng::from_seed(generator.seed);
    rng.set_stream(configuration.step);
    let value = match configuration.range {
      RandomConfiguredRange::Integer { min, max } => {
        RandomGeneratedValue::Integer(sample_i64_inclusive(&mut rng, min, max))
      }
      RandomConfiguredRange::Float { min, max } => {
        RandomGeneratedValue::Float(sample_f64_inclusive(&mut rng, min, max))
      }
    };
    generator.configuration = Some(RandomConfiguration {
      step: configuration.step.saturating_add(1),
      ..configuration
    });
    Some(value)
  }

  pub fn int_range_inclusive(
    &self,
    generators: &mut RandomGeneratorObjects,
    id: RandomGeneratorId,
    min: i64,
    max: i64,
  ) -> Option<i64> {
    if min > max {
      return None;
    }
    let generator = generators.generators.get_mut(&id)?;
    generator.draw_count = generator.draw_count.saturating_add(1);
    Some(sample_i64_inclusive(&mut generator.rng, min, max))
  }

  pub fn float_range_inclusive(
    &self,
    generators: &mut RandomGeneratorObjects,
    id: RandomGeneratorId,
    min: f64,
    max: f64,
  ) -> Option<f64> {
    if !min.is_finite() || !max.is_finite() || min > max {
      return None;
    }
    let generator = generators.generators.get_mut(&id)?;
    generator.draw_count = generator.draw_count.saturating_add(1);
    Some(sample_f64_inclusive(&mut generator.rng, min, max))
  }

  pub fn remove(&self, generators: &mut RandomGeneratorObjects, id: RandomGeneratorId) -> bool {
    generators.generators.remove(&id).is_some()
  }

  pub fn exists(&self, generators: &RandomGeneratorObjects, id: RandomGeneratorId) -> bool {
    generators.generators.contains_key(&id)
  }

  pub fn reseed(
    &self,
    generators: &mut RandomGeneratorObjects,
    id: RandomGeneratorId,
    seed: RandomSeed,
  ) -> bool {
    let Some(generator) = generators.generators.get_mut(&id) else {
      return false;
    };
    generator.reseed(seed);
    true
  }

  pub fn set_stream(
    &self,
    generators: &mut RandomGeneratorObjects,
    id: RandomGeneratorId,
    stream: u64,
  ) -> bool {
    let Some(generator) = generators.generators.get_mut(&id) else {
      return false;
    };
    generator.set_stream(stream);
    true
  }

  pub fn next_u32(&self, generators: &mut RandomGeneratorObjects, id: RandomGeneratorId) -> Option<u32> {
    let generator = generators.generators.get_mut(&id)?;
    generator.draw_count += 1;
    Some(generator.rng.next_u32())
  }

  pub fn next_u64(&self, generators: &mut RandomGeneratorObjects, id: RandomGeneratorId) -> Option<u64> {
    let generator = generators.generators.get_mut(&id)?;
    generator.draw_count += 1;
    Some(generator.rng.next_u64())
  }

  pub fn float_01(&self, generators: &mut RandomGeneratorObjects, id: RandomGeneratorId) -> Option<f64> {
    let generator = generators.generators.get_mut(&id)?;
    generator.draw_count += 1;
    Some(next_f64(&mut generator.rng))
  }

  pub fn int_range(
    &self,
    generators: &mut RandomGeneratorObjects,
    id: RandomGeneratorId,
    min: i64,
    max: i64,
  ) -> Option<i64> {
    if min >= max {
      return None;
    }
    let generator = generators.generators.get_mut(&id)?;
    generator.draw_count += 1;
    Some(sample_i64_range(&mut generator.rng, min, max))
  }

  pub fn bool(
    &self,
    generators: &mut RandomGeneratorObjects,
    id: RandomGeneratorId,
    probability: f64,
  ) -> Option<bool> {
    if probability.is_nan() {
      return None;
    }
    let generator = generators.generators.get_mut(&id)?;
    generator.draw_count += 1;
    if probability <= 0.0 {
      return Some(false);
    }
    if probability >= 1.0 {
      return Some(true);
    }
    Some(next_f64(&mut generator.rng) < probability)
  }

  pub fn snapshot(
    &self,
    generators: &RandomGeneratorObjects,
    id: RandomGeneratorId,
  ) -> Option<RandomSnapshot> {
    generators
      .generators
      .get(&id)
      .map(|generator| generator.snapshot(id))
  }

  pub fn restore(
    &self,
    generators: &mut RandomGeneratorObjects,
    snapshot: RandomSnapshot,
  ) -> RandomGeneratorId {
    generators
      .create(RandomGenerator::from_snapshot(&snapshot))
  }
}

impl Default for RandomService {
  fn default() -> Self {
    Self::new()
  }
}

fn next_f64(rng: &mut impl Rng) -> f64 {
  const SCALE: f64 = 1.0 / ((1u64 << 53) as f64);
  ((rng.next_u64() >> 11) as f64) * SCALE
}

fn sample_i64_range(rng: &mut impl Rng, min: i64, max: i64) -> i64 {
  let range = (max as i128 - min as i128) as u128;
  let zone = (u64::MAX as u128 + 1) - ((u64::MAX as u128 + 1) % range);
  loop {
    let value = rng.next_u64() as u128;
    if value < zone {
      return (min as i128 + (value % range) as i128) as i64;
    }
  }
}

fn sample_i64_inclusive(rng: &mut impl Rng, min: i64, max: i64) -> i64 {
  debug_assert!(min <= max);
  let range = (max as i128 - min as i128 + 1) as u128;
  if range == (u64::MAX as u128 + 1) {
    return (rng.next_u64() as i128 + i64::MIN as i128) as i64;
  }
  let zone = (u64::MAX as u128 + 1) - ((u64::MAX as u128 + 1) % range);
  loop {
    let value = rng.next_u64() as u128;
    if value < zone {
      return (min as i128 + (value % range) as i128) as i64;
    }
  }
}

fn sample_f64_inclusive(rng: &mut impl Rng, min: f64, max: f64) -> f64 {
  if min == max {
    return min;
  }
  let unit = rng.next_u64() as f64 / u64::MAX as f64;
  min * (1.0 - unit) + max * unit
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_exists_and_remove_generator() {
    let random = RandomService::new();
    let mut generators = RandomGeneratorObjects::new();

    let first = random.create(&mut generators, RandomSeed::U64(1));
    let second = random.create(&mut generators, RandomSeed::U64(2));

    assert_ne!(first, second);
    assert!(random.exists(&generators, first));
    assert!(random.remove(&mut generators, first));
    assert!(!random.exists(&generators, first));
  }

  #[test]
  fn same_seed_replays_same_sequence() {
    let random = RandomService::new();
    let mut left = RandomGeneratorObjects::new();
    let mut right = RandomGeneratorObjects::new();
    let a = random.create(&mut left, RandomSeed::U64(42));
    let b = random.create(&mut right, RandomSeed::U64(42));

    let left_values = (0..8)
      .map(|_| random.next_u64(&mut left, a).unwrap())
      .collect::<Vec<_>>();
    let right_values = (0..8)
      .map(|_| random.next_u64(&mut right, b).unwrap())
      .collect::<Vec<_>>();

    assert_eq!(left_values, right_values);
  }

  #[test]
  fn different_stream_changes_sequence() {
    let random = RandomService::new();
    let mut generators = RandomGeneratorObjects::new();
    let first = random.create(&mut generators, RandomSeed::U64(42));
    let second = random.create(&mut generators, RandomSeed::U64(42));
    assert!(random.set_stream(&mut generators, second, 7));

    assert_ne!(
      random.next_u64(&mut generators, first),
      random.next_u64(&mut generators, second)
    );
  }

  #[test]
  fn ranges_and_probability_edges_are_checked() {
    let random = RandomService::new();
    let mut generators = RandomGeneratorObjects::new();
    let id = random.create(&mut generators, RandomSeed::U64(3));

    for _ in 0..64 {
      let value = random.int_range(&mut generators, id, -5, 5).unwrap();
      assert!((-5..5).contains(&value));
    }
    assert_eq!(random.int_range(&mut generators, id, 5, 5), None);
    assert_eq!(random.bool(&mut generators, id, 0.0), Some(false));
    assert_eq!(random.bool(&mut generators, id, 1.0), Some(true));
    assert_eq!(random.bool(&mut generators, id, f64::NAN), None);
  }

  #[test]
  fn float_01_is_inside_half_open_unit_range() {
    let random = RandomService::new();
    let mut generators = RandomGeneratorObjects::new();
    let id = random.create(&mut generators, RandomSeed::U64(9));

    for _ in 0..64 {
      let value = random.float_01(&mut generators, id).unwrap();
      assert!((0.0..1.0).contains(&value));
    }
  }

  #[test]
  fn inclusive_float_range_stays_finite_for_extreme_opposite_bounds() {
    let random = RandomService::new();
    let mut generators = RandomGeneratorObjects::new();
    let id = random.create(&mut generators, RandomSeed::U64(19));

    for _ in 0..64 {
      let value = random
        .float_range_inclusive(&mut generators, id, -f64::MAX, f64::MAX)
        .unwrap();
      assert!(value.is_finite());
      assert!((-f64::MAX..=f64::MAX).contains(&value));
    }
  }

  #[test]
  fn reseed_restores_new_seed_start() {
    let random = RandomService::new();
    let mut generators = RandomGeneratorObjects::new();
    let id = random.create(&mut generators, RandomSeed::U64(1));
    let reference = random.create(&mut generators, RandomSeed::U64(2));

    assert!(random.reseed(&mut generators, id, RandomSeed::U64(2)));

    assert_eq!(
      random.next_u64(&mut generators, id),
      random.next_u64(&mut generators, reference)
    );
  }

  #[test]
  fn snapshot_and_restore_continue_same_sequence() {
    let random = RandomService::new();
    let mut generators = RandomGeneratorObjects::new();
    let id = random.create(&mut generators, RandomSeed::U64(99));

    for _ in 0..5 {
      let _ = random.next_u64(&mut generators, id);
    }
    let snapshot = random.snapshot(&generators, id).unwrap();
    let expected = (0..6)
      .map(|_| random.next_u64(&mut generators, id).unwrap())
      .collect::<Vec<_>>();

    let restored = random.restore(&mut generators, snapshot);
    let actual = (0..6)
      .map(|_| random.next_u64(&mut generators, restored).unwrap())
      .collect::<Vec<_>>();

    assert_eq!(actual, expected);
  }
}
