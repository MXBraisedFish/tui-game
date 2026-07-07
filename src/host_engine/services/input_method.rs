use std::env;
use std::time::Duration;

const ASCII_IM_ENV: &str = "IM_GUARD_ASCII_IM";
const RECONCILE_INTERVAL: Duration = Duration::from_millis(750);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImPolicy {
  Free,
  ForceAscii,
}

trait InputMethodBackend: Send {
  fn list_input_methods(&mut self) -> Result<Vec<String>, String>;
  fn get_input_method(&mut self) -> Result<String, String>;
  fn set_input_method(&mut self, input_method: &str) -> Result<(), String>;

  fn get_ime_state(&mut self) -> Result<Option<bool>, String> {
    Ok(None)
  }

  fn set_ime_state(&mut self, _enabled: bool) -> Result<(), String> {
    Ok(())
  }
}

struct SystemInputMethodBackend;

impl InputMethodBackend for SystemInputMethodBackend {
  fn list_input_methods(&mut self) -> Result<Vec<String>, String> {
    im_switch::list_input_methods().map_err(|err| err.to_string())
  }

  fn get_input_method(&mut self) -> Result<String, String> {
    im_switch::get_input_method().map_err(|err| err.to_string())
  }

  fn set_input_method(&mut self, input_method: &str) -> Result<(), String> {
    im_switch::set_input_method(input_method).map_err(|err| err.to_string())
  }

  #[cfg(target_os = "windows")]
  fn get_ime_state(&mut self) -> Result<Option<bool>, String> {
    im_switch::get_ime_state()
      .map(Some)
      .map_err(|err| err.to_string())
  }

  #[cfg(target_os = "windows")]
  fn set_ime_state(&mut self, enabled: bool) -> Result<(), String> {
    im_switch::set_ime_state(enabled).map_err(|err| err.to_string())
  }
}

pub struct InputMethodService {
  backend: Box<dyn InputMethodBackend>,
  ascii_im: Option<String>,
  saved_im: Option<String>,
  saved_ime_state: Option<bool>,
  active: bool,
  policy: ImPolicy,
  reconcile_elapsed: Duration,
  last_error: Option<String>,
}

impl InputMethodService {
  pub fn new() -> Self {
    Self::from_backend(Box::new(SystemInputMethodBackend), read_ascii_im_override())
  }

  fn from_backend(
    mut backend: Box<dyn InputMethodBackend>,
    ascii_override: Option<String>,
  ) -> Self {
    let methods = backend.list_input_methods().unwrap_or_default();
    let ascii_im = resolve_ascii_input_method(&methods, ascii_override);
    let last_error = ascii_im.is_none().then(|| {
      format!(
        "unable to resolve ASCII input method; set {ASCII_IM_ENV} or install an English/ASCII input method"
      )
    });

    Self {
      backend,
      ascii_im,
      saved_im: None,
      saved_ime_state: None,
      active: false,
      policy: ImPolicy::Free,
      reconcile_elapsed: Duration::ZERO,
      last_error,
    }
  }

  pub fn set_policy(&mut self, policy: ImPolicy) -> bool {
    self.policy = policy;
    match policy {
      ImPolicy::Free => self.release(),
      ImPolicy::ForceAscii => self.restrict(),
    }
  }

  pub fn policy(&self) -> ImPolicy {
    self.policy
  }

  pub fn restrict_input_method(&mut self) -> bool {
    self.policy = ImPolicy::ForceAscii;
    self.restrict()
  }

  pub fn release_input_method(&mut self) -> bool {
    self.policy = ImPolicy::Free;
    self.release()
  }

  pub fn is_input_method_restricted(&self) -> bool {
    self.active
  }

  pub fn update(&mut self, dt: Duration) {
    if self.policy != ImPolicy::ForceAscii || !self.active {
      self.reconcile_elapsed = Duration::ZERO;
      return;
    }

    self.reconcile_elapsed += dt;
    if self.reconcile_elapsed >= RECONCILE_INTERVAL {
      self.reconcile_elapsed = Duration::ZERO;
      let _ = self.reconcile_now();
    }
  }

  pub fn reconcile_now(&mut self) -> bool {
    if !self.active {
      return true;
    }

    let Some(ascii_im) = self.ascii_im.clone() else {
      self.last_error = Some("ASCII input method is unavailable".to_string());
      return false;
    };

    let input_method_ok = match self.backend.get_input_method() {
      Ok(current) if current == ascii_im => true,
      Ok(_) => self.set_ascii_input_method(&ascii_im),
      Err(err) => {
        self.last_error = Some(format!("failed to get current input method: {err}"));
        false
      }
    };
    let ime_ok = self.ensure_ime_closed();

    if input_method_ok && ime_ok {
      self.last_error = None;
      true
    } else {
      false
    }
  }

  pub fn ascii_input_method(&self) -> Option<&str> {
    self.ascii_im.as_deref()
  }

  pub fn last_error(&self) -> Option<&str> {
    self.last_error.as_deref()
  }

  fn restrict(&mut self) -> bool {
    if self.active {
      return true;
    }

    let Some(ascii_im) = self.ascii_im.clone() else {
      self.last_error = Some("ASCII input method is unavailable".to_string());
      return false;
    };

    let current = match self.backend.get_input_method() {
      Ok(current) => current,
      Err(err) => {
        self.last_error = Some(format!("failed to get current input method: {err}"));
        return false;
      }
    };

    let ime_state = self.backend.get_ime_state().ok().flatten();

    if current != ascii_im && self.saved_im.is_none() {
      self.saved_im = Some(current);
    }
    if ime_state != Some(false) && self.saved_ime_state.is_none() {
      self.saved_ime_state = ime_state;
    }

    let switched = self.set_ascii_input_method(&ascii_im);
    let ime_closed = self.ensure_ime_closed();
    if switched {
      self.active = true;
      self.reconcile_elapsed = Duration::ZERO;
      switched && ime_closed
    } else {
      false
    }
  }

  fn release(&mut self) -> bool {
    if !self.active {
      return true;
    }

    let saved = self.saved_im.take();
    let saved_ime_state = self.saved_ime_state.take();

    if saved.is_none() && saved_ime_state.is_none() {
      self.active = false;
      self.reconcile_elapsed = Duration::ZERO;
      self.last_error = None;
      return true;
    };

    if let Some(saved) = saved {
      if let Err(err) = self.backend.set_input_method(&saved) {
        self.saved_im = Some(saved);
        self.saved_ime_state = saved_ime_state;
        self.last_error = Some(format!("failed to restore input method: {err}"));
        return false;
      }
    }

    if let Some(saved_ime_state) = saved_ime_state {
      if let Err(err) = self.backend.set_ime_state(saved_ime_state) {
        self.saved_ime_state = Some(saved_ime_state);
        self.last_error = Some(format!("failed to restore IME state: {err}"));
        false
      } else {
        self.active = false;
        self.reconcile_elapsed = Duration::ZERO;
        self.last_error = None;
        true
      }
    } else {
      self.active = false;
      self.reconcile_elapsed = Duration::ZERO;
      self.last_error = None;
      true
    }
  }

  fn set_ascii_input_method(&mut self, ascii_im: &str) -> bool {
    match self.backend.set_input_method(ascii_im) {
      Ok(()) => {
        self.last_error = None;
        true
      }
      Err(err) => {
        self.last_error = Some(format!("failed to switch to ASCII input method: {err}"));
        false
      }
    }
  }

  fn ensure_ime_closed(&mut self) -> bool {
    match self.backend.get_ime_state() {
      Ok(Some(true)) => match self.backend.set_ime_state(false) {
        Ok(()) => true,
        Err(err) => {
          self.last_error = Some(format!("failed to close IME: {err}"));
          false
        }
      },
      Ok(Some(false)) | Ok(None) => true,
      Err(err) => {
        self.last_error = Some(format!("failed to get IME state: {err}"));
        false
      }
    }
  }
}

impl Default for InputMethodService {
  fn default() -> Self {
    Self::new()
  }
}

impl Drop for InputMethodService {
  fn drop(&mut self) {
    let _ = self.release_input_method();
  }
}

fn read_ascii_im_override() -> Option<String> {
  env::var(ASCII_IM_ENV)
    .ok()
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
}

fn resolve_ascii_input_method(
  methods: &[String],
  ascii_override: Option<String>,
) -> Option<String> {
  if let Some(value) = ascii_override.filter(|value| !value.trim().is_empty()) {
    return Some(value);
  }

  let exact_candidates = [
    "com.apple.keylayout.ABC",
    "com.apple.keylayout.US",
    "00000409",
    "keyboard-us",
    "xkb:us::eng",
  ];
  for candidate in exact_candidates {
    if let Some(found) = methods.iter().find(|method| method.as_str() == candidate) {
      return Some(found.clone());
    }
  }

  let weak_keywords = [
    "abc",
    "ascii",
    "english",
    "keyboard-us",
    "keyboard",
    "xkb:us",
    "us",
    "en",
  ];
  for keyword in weak_keywords {
    if let Some(found) = methods
      .iter()
      .find(|method| method.to_lowercase().contains(keyword))
    {
      return Some(found.clone());
    }
  }

  None
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::{Arc, Mutex};

  #[derive(Default)]
  struct FakeState {
    methods: Vec<String>,
    current: String,
    ime_state: Option<bool>,
    set_calls: Vec<String>,
    ime_set_calls: Vec<bool>,
    get_error: Option<String>,
    set_error: Option<String>,
    ime_get_error: Option<String>,
    ime_set_error: Option<String>,
  }

  struct FakeBackend {
    state: Arc<Mutex<FakeState>>,
  }

  impl FakeBackend {
    fn new(state: Arc<Mutex<FakeState>>) -> Self {
      Self { state }
    }
  }

  impl InputMethodBackend for FakeBackend {
    fn list_input_methods(&mut self) -> Result<Vec<String>, String> {
      Ok(self.state.lock().unwrap().methods.clone())
    }

    fn get_input_method(&mut self) -> Result<String, String> {
      let state = self.state.lock().unwrap();
      if let Some(err) = &state.get_error {
        Err(err.clone())
      } else {
        Ok(state.current.clone())
      }
    }

    fn set_input_method(&mut self, input_method: &str) -> Result<(), String> {
      let mut state = self.state.lock().unwrap();
      if let Some(err) = &state.set_error {
        return Err(err.clone());
      }
      state.current = input_method.to_string();
      state.set_calls.push(input_method.to_string());
      Ok(())
    }

    fn get_ime_state(&mut self) -> Result<Option<bool>, String> {
      let state = self.state.lock().unwrap();
      if let Some(err) = &state.ime_get_error {
        Err(err.clone())
      } else {
        Ok(state.ime_state)
      }
    }

    fn set_ime_state(&mut self, enabled: bool) -> Result<(), String> {
      let mut state = self.state.lock().unwrap();
      if let Some(err) = &state.ime_set_error {
        return Err(err.clone());
      }
      state.ime_state = Some(enabled);
      state.ime_set_calls.push(enabled);
      Ok(())
    }
  }

  fn service_with_state(
    state: Arc<Mutex<FakeState>>,
    ascii_override: Option<String>,
  ) -> InputMethodService {
    InputMethodService::from_backend(Box::new(FakeBackend::new(state)), ascii_override)
  }

  #[test]
  fn ascii_input_method_can_be_overridden() {
    let state = Arc::new(Mutex::new(FakeState {
      methods: vec!["zh".to_string()],
      current: "zh".to_string(),
      ..Default::default()
    }));
    let service = service_with_state(state, Some("custom-ascii".to_string()));

    assert_eq!(service.ascii_input_method(), Some("custom-ascii"));
  }

  #[test]
  fn ascii_input_method_can_be_guessed_from_methods() {
    let state = Arc::new(Mutex::new(FakeState {
      methods: vec!["zh".to_string(), "00000409".to_string()],
      current: "zh".to_string(),
      ..Default::default()
    }));
    let service = service_with_state(state, None);

    assert_eq!(service.ascii_input_method(), Some("00000409"));
  }

  #[test]
  fn missing_ascii_input_method_degrades_without_panic() {
    let state = Arc::new(Mutex::new(FakeState {
      methods: vec!["zh".to_string()],
      current: "zh".to_string(),
      ..Default::default()
    }));
    let mut service = service_with_state(state, None);

    assert_eq!(service.ascii_input_method(), None);
    assert!(!service.restrict_input_method());
    assert!(service.last_error().is_some());
  }

  #[test]
  fn restrict_saves_original_and_switches_to_ascii_once() {
    let state = Arc::new(Mutex::new(FakeState {
      methods: vec!["zh".to_string(), "00000409".to_string()],
      current: "zh".to_string(),
      ime_state: Some(true),
      ..Default::default()
    }));
    let mut service = service_with_state(state.clone(), None);

    assert!(service.restrict_input_method());
    assert!(service.restrict_input_method());

    let state = state.lock().unwrap();
    assert_eq!(state.current, "00000409");
    assert_eq!(state.set_calls, vec!["00000409"]);
    assert_eq!(state.ime_state, Some(false));
    assert_eq!(state.ime_set_calls, vec![false]);
  }

  #[test]
  fn release_restores_saved_input_method() {
    let state = Arc::new(Mutex::new(FakeState {
      methods: vec!["zh".to_string(), "00000409".to_string()],
      current: "zh".to_string(),
      ime_state: Some(true),
      ..Default::default()
    }));
    let mut service = service_with_state(state.clone(), None);

    assert!(service.restrict_input_method());
    assert!(service.release_input_method());

    let state = state.lock().unwrap();
    assert_eq!(state.current, "zh");
    assert_eq!(state.ime_state, Some(true));
    assert_eq!(state.set_calls, vec!["00000409", "zh"]);
    assert_eq!(state.ime_set_calls, vec![false, true]);
  }

  #[test]
  fn failed_release_keeps_saved_input_method_for_retry() {
    let state = Arc::new(Mutex::new(FakeState {
      methods: vec!["zh".to_string(), "00000409".to_string()],
      current: "zh".to_string(),
      ..Default::default()
    }));
    let mut service = service_with_state(state.clone(), None);
    assert!(service.restrict_input_method());

    state.lock().unwrap().set_error = Some("restore failed".to_string());
    assert!(!service.release_input_method());
    assert!(service.is_input_method_restricted());

    state.lock().unwrap().set_error = None;
    assert!(service.release_input_method());
    assert_eq!(state.lock().unwrap().current, "zh");
  }

  #[test]
  fn reconcile_runs_only_after_interval() {
    let state = Arc::new(Mutex::new(FakeState {
      methods: vec!["zh".to_string(), "00000409".to_string()],
      current: "zh".to_string(),
      ime_state: Some(false),
      ..Default::default()
    }));
    let mut service = service_with_state(state.clone(), None);
    assert!(service.restrict_input_method());
    state.lock().unwrap().set_calls.clear();
    state.lock().unwrap().current = "zh".to_string();

    service.update(Duration::from_millis(749));
    assert!(state.lock().unwrap().set_calls.is_empty());

    service.update(Duration::from_millis(1));
    assert_eq!(state.lock().unwrap().set_calls, vec!["00000409"]);
  }

  #[test]
  fn reconcile_closes_ime_when_layout_is_already_ascii() {
    let state = Arc::new(Mutex::new(FakeState {
      methods: vec!["zh".to_string(), "00000409".to_string()],
      current: "zh".to_string(),
      ime_state: Some(false),
      ..Default::default()
    }));
    let mut service = service_with_state(state.clone(), None);
    assert!(service.restrict_input_method());
    state.lock().unwrap().ime_set_calls.clear();
    state.lock().unwrap().ime_state = Some(true);

    service.update(Duration::from_millis(750));

    let state = state.lock().unwrap();
    assert_eq!(state.current, "00000409");
    assert_eq!(state.ime_state, Some(false));
    assert_eq!(state.ime_set_calls, vec![false]);
  }

  #[test]
  fn drop_releases_restricted_input_method() {
    let state = Arc::new(Mutex::new(FakeState {
      methods: vec!["zh".to_string(), "00000409".to_string()],
      current: "zh".to_string(),
      ..Default::default()
    }));
    {
      let mut service = service_with_state(state.clone(), None);
      assert!(service.restrict_input_method());
    }

    assert_eq!(state.lock().unwrap().current, "zh");
  }
}
