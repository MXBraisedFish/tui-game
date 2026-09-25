//! Generational slot arena: stable `(index, generation)` handles that go stale on removal.

pub struct Arena<T> {
  slots: Vec<ArenaSlot<T>>,
}

struct ArenaSlot<T> {
  generation: u32,
  value: Option<T>,
}

impl<T> Arena<T> {
  pub fn new() -> Self {
    Self { slots: Vec::new() }
  }

  pub fn insert(&mut self, value: T) -> (u32, u32) {
    if let Some((index, slot)) = self
      .slots
      .iter_mut()
      .enumerate()
      .find(|(_, slot)| slot.value.is_none())
    {
      slot.value = Some(value);
      return (index as u32, slot.generation);
    }
    let index = self.slots.len() as u32;
    self.slots.push(ArenaSlot {
      generation: 1,
      value: Some(value),
    });
    (index, 1)
  }

  pub fn get(&self, index: u32, generation: u32) -> Option<&T> {
    let slot = self.slots.get(index as usize)?;
    (slot.generation == generation).then_some(slot.value.as_ref()?)
  }

  pub fn get_mut(&mut self, index: u32, generation: u32) -> Option<&mut T> {
    let slot = self.slots.get_mut(index as usize)?;
    (slot.generation == generation).then_some(slot.value.as_mut()?)
  }

  pub fn remove(&mut self, index: u32, generation: u32) -> Option<T> {
    let slot = self.slots.get_mut(index as usize)?;
    if slot.generation != generation {
      return None;
    }
    let value = slot.value.take()?;
    slot.generation = slot.generation.wrapping_add(1).max(1);
    Some(value)
  }

  pub fn keys(&self) -> Vec<(u32, u32)> {
    self
      .slots
      .iter()
      .enumerate()
      .filter_map(|(index, slot)| slot.value.as_ref().map(|_| (index as u32, slot.generation)))
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::Arena;

  #[test]
  fn removed_slots_are_reused_with_a_new_generation() {
    let mut arena = Arena::new();
    let first = arena.insert("a");
    let second = arena.insert("b");
    assert_eq!(first, (0, 1));
    assert_eq!(second, (1, 1));
    assert_eq!(arena.remove(first.0, first.1), Some("a"));
    assert_eq!(arena.get(first.0, first.1), None);
    assert_eq!(arena.remove(first.0, first.1), None);
    let reused = arena.insert("c");
    assert_eq!(reused, (0, 2));
    assert_eq!(arena.get(reused.0, reused.1), Some(&"c"));
    *arena.get_mut(second.0, second.1).unwrap() = "B";
    assert_eq!(arena.keys(), vec![(0, 2), (1, 1)]);
    assert_eq!(arena.get(1, 1), Some(&"B"));
  }
}
