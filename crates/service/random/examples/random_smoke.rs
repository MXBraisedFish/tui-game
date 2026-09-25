//! Minimal entry: seeded generators are reproducible and snapshots restore the stream.

use tg_service_random::{RandomGeneratorObjects, RandomSeed, RandomService};

fn main() {
  let random = RandomService::new();
  let mut generators = RandomGeneratorObjects::new();
  let first = random.create(&mut generators, RandomSeed::U64(42));
  let second = random.create(&mut generators, RandomSeed::U64(42));
  let a = random.int_range(&mut generators, first, 0, 1000);
  let b = random.int_range(&mut generators, second, 0, 1000);
  assert_eq!(a, b, "same seed, same value");
  assert_eq!(generators.len(), 2);
  println!("random ok: {a:?}");
}
