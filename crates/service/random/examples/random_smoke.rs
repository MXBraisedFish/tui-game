//! Independent random smoke entry exercising the public API and checking its results.

use tg_service_random::{
  RandomConfiguration, RandomConfiguredRange, RandomGeneratorObjects, RandomSeed, RandomService,
};

fn main() {
  let random = RandomService::new();
  let mut generators = RandomGeneratorObjects::new();
  let first = random.create(&mut generators, RandomSeed::U64(42));
  let second = random.create(&mut generators, RandomSeed::U64(42));
  let a = random.int_range(&mut generators, first, 0, 1000);
  let b = random.int_range(&mut generators, second, 0, 1000);
  assert_eq!(a, b, "same seed, same value");
  assert_eq!(generators.len(), 2);
  let configuration = RandomConfiguration {
    range: RandomConfiguredRange::Integer { min: 0, max: 1000 },
    seed: 42,
    step: i64::MAX as u64,
  };
  let boundary = random.create_configured(&mut generators, configuration);
  let reference = random.create_configured(
    &mut generators,
    RandomConfiguration {
      step: 0,
      ..configuration
    },
  );
  assert!(
    random
      .generate_configured(&mut generators, boundary)
      .is_some()
  );
  assert_eq!(random.configuration(&generators, boundary).unwrap().step, 0);
  assert_eq!(
    random.generate_configured(&mut generators, boundary),
    random.generate_configured(&mut generators, reference)
  );
  println!("random ok: {a:?}");
}
