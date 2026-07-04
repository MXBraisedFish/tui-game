use std::collections::HashMap;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RandomGeneratorId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RandomAlgorithm {
  ChaCha8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RandomSeed {
  U64(u64),
  Bytes32([u8; 32]),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RandomSnapshot {
  pub id: RandomGeneratorId,
  pub algorithm: RandomAlgorithm,
  pub seed: [u8; 32],
  pub stream: u64,
  pub word_pos: u128,
  pub draw_count: u64,
}

pub(crate) struct RandomGenerator {
  pub(crate) algorithm: RandomAlgorithm,
  pub(crate) seed: [u8; 32],
  pub(crate) stream: u64,
  pub(crate) draw_count: u64,
  pub(crate) rng: ChaCha8Rng,
}

impl RandomGenerator {
  pub(crate) fn new(seed: RandomSeed) -> Self {
    Self::from_parts(seed_to_bytes(seed), 0, 0, 0)
  }

  pub(crate) fn from_snapshot(snapshot: &RandomSnapshot) -> Self {
    Self::from_parts(
      snapshot.seed,
      snapshot.stream,
      snapshot.word_pos,
      snapshot.draw_count,
    )
  }

  pub(crate) fn reseed(&mut self, seed: RandomSeed) {
    *self = Self::new(seed);
  }

  pub(crate) fn set_stream(&mut self, stream: u64) {
    *self = Self::from_parts(self.seed, stream, 0, 0);
  }

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
    }
  }
}

pub(crate) struct RandomGeneratorObjects {
  pub(crate) next_id: u64,
  pub(crate) generators: HashMap<RandomGeneratorId, RandomGenerator>,
}

impl RandomGeneratorObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      generators: HashMap::new(),
    }
  }

  pub(crate) fn create(&mut self, generator: RandomGenerator) -> RandomGeneratorId {
    let id = RandomGeneratorId(self.next_id);
    self.next_id += 1;
    self.generators.insert(id, generator);
    id
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
