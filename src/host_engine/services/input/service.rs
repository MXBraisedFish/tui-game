use std::collections::HashSet;
use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};
use std::thread;

use crossbeam_channel::{Receiver, Sender, unbounded};

use rdev::{Event, EventType, Key, listen};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEventKind {
  Press,
  Release,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
  pub key: Key,
  pub kind: KeyEventKind,
}

pub struct InputService {
  sender: Sender<KeyEvent>,
  receiver: Receiver<KeyEvent>,
  listener_started: Arc<AtomicBool>,
  held_keys: HashSet<Key>,
  pressed_keys: HashSet<Key>,
  released_keys: HashSet<Key>,
}

impl InputService {
  pub fn new() -> Self {
    let (sender, receiver) = unbounded();
    Self {
      sender,
      receiver,
      listener_started: Arc::new(AtomicBool::new(false)),
      held_keys: HashSet::new(),
      pressed_keys: HashSet::new(),
      released_keys: HashSet::new(),
    }
  }

  pub fn start_key_listener(&self) {
    if self.listener_started.swap(true, Ordering::SeqCst) {
      return;
    }

    let sender = self.sender.clone();

    thread::spawn(move || {
      let callback = move |event: Event| {
        if let Some(key_event) = key_event_from_rdev(event) {
          let _ = sender.send(key_event);
        }
      };
      let _ = listen(callback);
    });
  }

  pub fn begin_frame(&mut self) {
    self.pressed_keys.clear();
    self.released_keys.clear();
  }

  pub fn poll(&mut self) {
    while let Ok(event) = self.receiver.try_recv() {
      self.apply_key_event(event);
    }
  }

  pub fn is_down(&self, key: Key) -> bool {
    self.held_keys.contains(&key)
  }

  pub fn was_pressed(&self, key: Key) -> bool {
    self.pressed_keys.contains(&key)
  }

  pub fn was_released(&self, key: Key) -> bool {
    self.released_keys.contains(&key)
  }

  pub fn clear(&mut self) {
    self.held_keys.clear();
    self.pressed_keys.clear();
    self.released_keys.clear();
  }

  fn apply_key_event(&mut self, event: KeyEvent) {
    match event.kind {
      KeyEventKind::Press => {
        self.held_keys.insert(event.key);
        self.pressed_keys.insert(event.key);
      }
      KeyEventKind::Release => {
        self.held_keys.remove(&event.key);
        self.released_keys.insert(event.key);
      }
    }
  }
}

fn key_event_from_rdev(event: Event) -> Option<KeyEvent> {
  match event.event_type {
    EventType::KeyPress(key) => Some(KeyEvent { key, kind: KeyEventKind::Press }),
    EventType::KeyRelease(key) => Some(KeyEvent { key, kind: KeyEventKind::Release }),
    _ => None,
  }
}
