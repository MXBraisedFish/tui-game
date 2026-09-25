//! Random service: seeded, snapshot-able ChaCha8 generators and configured value ranges.

mod objects;
mod service;

pub use objects::{
  RandomAlgorithm, RandomConfiguration, RandomConfiguredRange, RandomGeneratedValue,
  RandomGeneratorId, RandomGeneratorObjects, RandomSeed, RandomSnapshot,
};
pub use service::RandomService;
