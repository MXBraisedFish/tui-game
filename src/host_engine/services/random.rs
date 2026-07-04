use rand::Rng;

use super::widget::runtime_object::RuntimeObjectPool;
use super::widget::runtime_object::random::{
  RandomGenerator, RandomGeneratorId, RandomSeed, RandomSnapshot,
};

pub struct RandomService;

impl RandomService {
  pub fn new() -> Self {
    Self
  }

  pub fn create(&self, pool: &mut RuntimeObjectPool, seed: RandomSeed) -> RandomGeneratorId {
    pool.random_generators.create(RandomGenerator::new(seed))
  }

  pub fn remove(&self, pool: &mut RuntimeObjectPool, id: RandomGeneratorId) -> bool {
    pool.random_generators.generators.remove(&id).is_some()
  }

  pub fn exists(&self, pool: &RuntimeObjectPool, id: RandomGeneratorId) -> bool {
    pool.random_generators.generators.contains_key(&id)
  }

  pub fn reseed(
    &self,
    pool: &mut RuntimeObjectPool,
    id: RandomGeneratorId,
    seed: RandomSeed,
  ) -> bool {
    let Some(generator) = pool.random_generators.generators.get_mut(&id) else {
      return false;
    };
    generator.reseed(seed);
    true
  }

  pub fn set_stream(
    &self,
    pool: &mut RuntimeObjectPool,
    id: RandomGeneratorId,
    stream: u64,
  ) -> bool {
    let Some(generator) = pool.random_generators.generators.get_mut(&id) else {
      return false;
    };
    generator.set_stream(stream);
    true
  }

  pub fn next_u32(&self, pool: &mut RuntimeObjectPool, id: RandomGeneratorId) -> Option<u32> {
    let generator = pool.random_generators.generators.get_mut(&id)?;
    generator.draw_count += 1;
    Some(generator.rng.next_u32())
  }

  pub fn next_u64(&self, pool: &mut RuntimeObjectPool, id: RandomGeneratorId) -> Option<u64> {
    let generator = pool.random_generators.generators.get_mut(&id)?;
    generator.draw_count += 1;
    Some(generator.rng.next_u64())
  }

  pub fn float_01(&self, pool: &mut RuntimeObjectPool, id: RandomGeneratorId) -> Option<f64> {
    let generator = pool.random_generators.generators.get_mut(&id)?;
    generator.draw_count += 1;
    Some(next_f64(&mut generator.rng))
  }

  pub fn int_range(
    &self,
    pool: &mut RuntimeObjectPool,
    id: RandomGeneratorId,
    min: i64,
    max: i64,
  ) -> Option<i64> {
    if min >= max {
      return None;
    }
    let generator = pool.random_generators.generators.get_mut(&id)?;
    generator.draw_count += 1;
    Some(sample_i64_range(&mut generator.rng, min, max))
  }

  pub fn bool(
    &self,
    pool: &mut RuntimeObjectPool,
    id: RandomGeneratorId,
    probability: f64,
  ) -> Option<bool> {
    if probability.is_nan() {
      return None;
    }
    let generator = pool.random_generators.generators.get_mut(&id)?;
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
    pool: &RuntimeObjectPool,
    id: RandomGeneratorId,
  ) -> Option<RandomSnapshot> {
    pool
      .random_generators
      .generators
      .get(&id)
      .map(|generator| generator.snapshot(id))
  }

  pub fn restore(
    &self,
    pool: &mut RuntimeObjectPool,
    snapshot: RandomSnapshot,
  ) -> RandomGeneratorId {
    pool
      .random_generators
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_exists_and_remove_generator() {
    let random = RandomService::new();
    let mut pool = RuntimeObjectPool::new();

    let first = random.create(&mut pool, RandomSeed::U64(1));
    let second = random.create(&mut pool, RandomSeed::U64(2));

    assert_ne!(first, second);
    assert!(random.exists(&pool, first));
    assert!(random.remove(&mut pool, first));
    assert!(!random.exists(&pool, first));
  }

  #[test]
  fn same_seed_replays_same_sequence() {
    let random = RandomService::new();
    let mut left = RuntimeObjectPool::new();
    let mut right = RuntimeObjectPool::new();
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
    let mut pool = RuntimeObjectPool::new();
    let first = random.create(&mut pool, RandomSeed::U64(42));
    let second = random.create(&mut pool, RandomSeed::U64(42));
    assert!(random.set_stream(&mut pool, second, 7));

    assert_ne!(
      random.next_u64(&mut pool, first),
      random.next_u64(&mut pool, second)
    );
  }

  #[test]
  fn ranges_and_probability_edges_are_checked() {
    let random = RandomService::new();
    let mut pool = RuntimeObjectPool::new();
    let id = random.create(&mut pool, RandomSeed::U64(3));

    for _ in 0..64 {
      let value = random.int_range(&mut pool, id, -5, 5).unwrap();
      assert!((-5..5).contains(&value));
    }
    assert_eq!(random.int_range(&mut pool, id, 5, 5), None);
    assert_eq!(random.bool(&mut pool, id, 0.0), Some(false));
    assert_eq!(random.bool(&mut pool, id, 1.0), Some(true));
    assert_eq!(random.bool(&mut pool, id, f64::NAN), None);
  }

  #[test]
  fn float_01_is_inside_half_open_unit_range() {
    let random = RandomService::new();
    let mut pool = RuntimeObjectPool::new();
    let id = random.create(&mut pool, RandomSeed::U64(9));

    for _ in 0..64 {
      let value = random.float_01(&mut pool, id).unwrap();
      assert!((0.0..1.0).contains(&value));
    }
  }

  #[test]
  fn reseed_restores_new_seed_start() {
    let random = RandomService::new();
    let mut pool = RuntimeObjectPool::new();
    let id = random.create(&mut pool, RandomSeed::U64(1));
    let reference = random.create(&mut pool, RandomSeed::U64(2));

    assert!(random.reseed(&mut pool, id, RandomSeed::U64(2)));

    assert_eq!(
      random.next_u64(&mut pool, id),
      random.next_u64(&mut pool, reference)
    );
  }

  #[test]
  fn snapshot_and_restore_continue_same_sequence() {
    let random = RandomService::new();
    let mut pool = RuntimeObjectPool::new();
    let id = random.create(&mut pool, RandomSeed::U64(99));

    for _ in 0..5 {
      let _ = random.next_u64(&mut pool, id);
    }
    let snapshot = random.snapshot(&pool, id).unwrap();
    let expected = (0..6)
      .map(|_| random.next_u64(&mut pool, id).unwrap())
      .collect::<Vec<_>>();

    let restored = random.restore(&mut pool, snapshot);
    let actual = (0..6)
      .map(|_| random.next_u64(&mut pool, restored).unwrap())
      .collect::<Vec<_>>();

    assert_eq!(actual, expected);
  }
}
