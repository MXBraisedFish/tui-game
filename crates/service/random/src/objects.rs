//! Objects support for the random service.

use std::collections::HashMap;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// The identity of random generator within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RandomGeneratorId(
  /// The wrapped u64 value.
  pub u64,
);

/// The supported algorithm used by an owned random generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RandomAlgorithm {
  /// The cha cha8 setting for random algorithm.
  ChaCha8,
}

/// A seed value or seed-selection rule for generator initialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RandomSeed {
  /// The u64 setting for random seed.
  U64(u64),
  /// The bytes32 setting for random seed.
  Bytes32([u8; 32]),
}

/// A retained copy of random data for later inspection or replay.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `algorithm` - The algorithm.
/// * `seed` - The seed initializing the random stream.
/// * `stream` - The random stream selector.
/// * `word_pos` - The word pos.
/// * `draw_count` - The draw count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RandomSnapshot {
  /// The identifier of the owned object.
  pub id: RandomGeneratorId,
  /// The algorithm.
  pub algorithm: RandomAlgorithm,
  /// The seed initializing the random stream.
  pub seed: [u8; 32],
  /// The random stream selector.
  pub stream: u64,
  /// The word pos.
  pub word_pos: u128,
  /// The draw count.
  pub draw_count: u64,
}

/// The bounded distribution selected for configured random generation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RandomConfiguredRange {
  /// The integer setting for random configured range.
  Integer {
    /// The lower bound of the accepted or generated range.
    min: i64,
    /// The upper bound of the accepted or generated range.
    max: i64,
  },
  /// The float setting for random configured range.
  Float {
    /// The lower bound of the accepted or generated range.
    min: f64,
    /// The upper bound of the accepted or generated range.
    max: f64,
  },
}

/// An algorithm, seed, range, and initial stream position for an owned generator.
///
/// # Fields
///
/// * `range` - The range.
/// * `seed` - The seed initializing the random stream.
/// * `step` - The step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RandomConfiguration {
  /// The range.
  pub range: RandomConfiguredRange,
  /// The seed initializing the random stream.
  pub seed: i64,
  /// The step.
  pub step: u64,
}

/// An integer, floating-point, or boolean result from configured random generation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RandomGeneratedValue {
  /// The integer setting for random generated value.
  Integer(i64),
  /// The float setting for random generated value.
  Float(f64),
}

/// The random generator representation used by this module.
///
/// # Fields
///
/// * `algorithm` - The algorithm.
/// * `seed` - The seed initializing the random stream.
/// * `stream` - The random stream selector.
/// * `draw_count` - The draw count.
/// * `rng` - The rng.
/// * `configuration` - The configuration.
pub(crate) struct RandomGenerator {
  /// The algorithm.
  pub(crate) algorithm: RandomAlgorithm,
  /// The seed initializing the random stream.
  pub(crate) seed: [u8; 32],
  /// The random stream selector.
  pub(crate) stream: u64,
  /// The draw count.
  pub(crate) draw_count: u64,
  /// The rng.
  pub(crate) rng: ChaCha8Rng,
  /// The configuration.
  pub(crate) configuration: Option<RandomConfiguration>,
}

impl RandomGenerator {
  /// Create a random generator initialized from `seed`.
  pub(crate) fn new(seed: RandomSeed) -> Self {
    Self::from_parts(seed_to_bytes(seed), 0, 0, 0)
  }

  /// Create a random generator from its persisted algorithm, seed, stream, and state.
  pub(crate) fn from_snapshot(snapshot: &RandomSnapshot) -> Self {
    Self::from_parts(
      snapshot.seed,
      snapshot.stream,
      snapshot.word_pos,
      snapshot.draw_count,
    )
  }

  /// Reset the generator's stream state using the supplied seed.
  pub(crate) fn reseed(&mut self, seed: RandomSeed) {
    *self = Self::new(seed);
  }

  /// Update the stream used by this random generator.
  pub(crate) fn set_stream(&mut self, stream: u64) {
    *self = Self::from_parts(self.seed, stream, 0, 0);
  }

  /// Return the snapshot for the addressed object.
  pub(crate) fn snapshot(&self, id: RandomGeneratorId) -> RandomSnapshot {
    RandomSnapshot {
      id,
      algorithm: self.algorithm,
      seed: self.seed,
      stream: self.stream,
      word_pos: self.rng.get_word_pos(),
      draw_count: self.draw_count,
    }
  }

  fn from_parts(seed: [u8; 32], stream: u64, word_pos: u128, draw_count: u64) -> Self {
    let mut rng = ChaCha8Rng::from_seed(seed);
    rng.set_stream(stream);
    rng.set_word_pos(word_pos);
    Self {
      algorithm: RandomAlgorithm::ChaCha8,
      seed,
      stream,
      draw_count,
      rng,
      configuration: None,
    }
  }
}

/// The collection of owned random generator instances and their queued events.
///
/// # Fields
///
/// * `next_id` - The identifier of the next.
/// * `generators` - The generators indexed by their declared keys.
pub struct RandomGeneratorObjects {
  /// The identifier of the next.
  pub(crate) next_id: u64,
  /// The generators indexed by their declared keys.
  pub(crate) generators: HashMap<RandomGeneratorId, RandomGenerator>,
}

impl RandomGeneratorObjects {
  /// Create a random generator objects with its initial state.
  pub fn new() -> Self {
    Self {
      next_id: 1,
      generators: HashMap::new(),
    }
  }

  /// Return the current len.
  pub fn len(&self) -> usize {
    self.generators.len()
  }

  /// Report whether this random generator objects is empty.
  pub fn is_empty(&self) -> bool {
    self.generators.is_empty()
  }

  /// Create an owned random generator objects object and return its identity.
  pub(crate) fn create(&mut self, generator: RandomGenerator) -> RandomGeneratorId {
    let id = RandomGeneratorId(self.next_id);
    self.next_id += 1;
    self.generators.insert(id, generator);
    id
  }
}

impl Default for RandomGeneratorObjects {
  fn default() -> Self {
    Self::new()
  }
}

fn seed_to_bytes(seed: RandomSeed) -> [u8; 32] {
  match seed {
    RandomSeed::Bytes32(bytes) => bytes,
    RandomSeed::U64(value) => {
      let mut state = value;
      let mut bytes = [0; 32];
      for chunk in bytes.chunks_exact_mut(8) {
        state = splitmix64(state);
        chunk.copy_from_slice(&state.to_le_bytes());
      }
      bytes
    }
  }
}

fn splitmix64(mut value: u64) -> u64 {
  value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
  let mut z = value;
  z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
  z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
  z ^ (z >> 31)
}
