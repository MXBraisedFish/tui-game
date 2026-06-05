use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};
use std::thread;

use crossterm::event::KeyModifiers;
use rdev::{Event, EventType, listen};

use super::key_code_from_rdev;
use super::{ExternalRawInputSender, KeyboardInputEvent, KeyboardInputKind, RawInputEvent, RawInputSource};

#[derive(Clone)]
pub struct GlobalKeyboardControl {
  enabled: Arc<AtomicBool>,
}

impl GlobalKeyboardControl {
  pub fn new() -> Self {
    Self {
      enabled: Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn enable(&self) {
    self.enabled.store(true, Ordering::SeqCst);
  }

  pub fn disable(&self) {
    self.enabled.store(false, Ordering::SeqCst);
  }

  pub fn is_enabled(&self) -> bool {
    self.enabled.load(Ordering::SeqCst)
  }
}

pub struct GlobalKeyboardListener {
  control: GlobalKeyboardControl,
  started: bool,
}

impl GlobalKeyboardListener {
  pub fn new() -> Self {
    Self {
      control: GlobalKeyboardControl::new(),
      started: false,
    }
  }

  pub fn control(&self) -> GlobalKeyboardControl {
    self.control.clone()
  }

  pub fn is_started(&self) -> bool {
    self.started
  }

  pub fn start(&mut self, sender: ExternalRawInputSender) {
    if self.started {
      return;
    }

    self.started = true;

    let control = self.control.clone();

    thread::spawn(move || {
      let callback = move |event: Event| {
        if !control.is_enabled() {
          return;
        }

        if let Some(raw_event) = raw_event_from_rdev(event) {
          sender.push(raw_event);
        }
      };

      let _ = listen(callback);
    });
  }

  pub fn enable(&self) {
    self.control.enable();
  }

  pub fn disable(&self) {
    self.control.disable();
  }
}

fn raw_event_from_rdev(event: Event) -> Option<RawInputEvent> {
  match event.event_type {
    EventType::KeyPress(key) => {
      key_code_from_rdev(key).map(|code| {
        RawInputEvent::Keyboard {
          source: RawInputSource::GlobalKeyboard,
          event: KeyboardInputEvent::new(code, KeyModifiers::empty(), KeyboardInputKind::Press),
        }
      })
    }
    EventType::KeyRelease(key) => {
      key_code_from_rdev(key).map(|code| {
        RawInputEvent::Keyboard {
          source: RawInputSource::GlobalKeyboard,
          event: KeyboardInputEvent::new(code, KeyModifiers::empty(), KeyboardInputKind::Release),
        }
      })
    }
    _ => None,
  }
}
