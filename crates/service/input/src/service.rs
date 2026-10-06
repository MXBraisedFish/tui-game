//! Service support for the input service.

use std::cell::Cell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, unbounded};

use crossterm::event::{
  self as ct_event, Event as CtEvent, KeyCode as CtKeyCode, KeyEvent as CtKeyEvent,
  KeyEventKind as CtKeyEventKind, KeyModifiers as CtKeyModifiers, MouseEvent as CtMouseEvent,
  MouseEventKind as CtMouseEventKind,
};
use rdev::{Event, EventType, Key as RdevKey, listen};

use tg_core_log::LogSource;
use tg_service_async::AsyncRuntime;
use tg_service_log::LogService;

use tg_core_input::{
  FocusEvent, InputActionEvent, InputEventType, Key, KeyBinding, KeyEvent, KeyEventKind,
  KeyPattern, KeyState, MouseButton, MouseEvent, MouseEventKind, RawKeyEvent, ResizeEvent,
  ScrollDirection, SystemEvent, TerminalKeyCode, TerminalKeyEvent, display_key_token,
};

/// Failures reported by input listener operations.
///
/// # Fields
///
/// * `0` - The wrapped string value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputListenerError(
  /// The wrapped string value.
  pub String,
);

/// Text committed by the terminal, without input-method preedit information.
///
/// # Fields
///
/// * `text` - Submitted characters or pasted text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommittedTextEvent {
  /// Submitted characters or pasted text.
  pub text: String,
}

/// An ordered keyboard state change or focus boundary observed during one input frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputNotification {
  /// A physical key transition, independent of shortcut capture.
  Key { key: Key, state: KeyState },
  /// A logical action transition and its host ownership.
  Action {
    event: InputActionEvent,
    system: bool,
  },
  /// Committed text observed independently of shortcut and widget capture.
  Text { text: String },
  /// A terminal focus boundary following any closing releases.
  Focus { gained: bool },
}

#[derive(Clone, Copy)]
struct ActiveAction {
  state: KeyState,
  frame: u64,
  priority: u64,
  combination: bool,
  registration: usize,
}

enum QueuedInput {
  Key(KeyEvent),
  Focus(FocusEvent),
  Text(CommittedTextEvent),
}

/// The public entry point for input operations.
pub struct InputService {
  sender: Sender<QueuedInput>,
  receiver: Receiver<QueuedInput>,
  action_sender: Sender<InputActionEvent>,
  action_receiver: Receiver<InputActionEvent>,
  system_sender: Sender<SystemEvent>,
  system_receiver: Receiver<SystemEvent>,
  pending_system_events: VecDeque<SystemEvent>,
  observed_system_events: VecDeque<SystemEvent>,
  key_listener_started: Arc<AtomicBool>,
  system_listener_started: Arc<AtomicBool>,
  held_keys: HashSet<Key>,
  blocked_keys: HashSet<Key>,
  blocked_page_keys: HashSet<Key>,
  blocked_system_keys: HashSet<Key>,
  key_transitions: HashMap<Key, (KeyState, u64)>,
  active_actions: HashMap<(bool, String), ActiveAction>,
  notifications: Vec<InputNotification>,
  frame: u64,
  dispatch_cursor: Cell<usize>,
  pressed_keys: HashSet<Key>,
  released_keys: HashSet<Key>,
  mouse_held_buttons: HashSet<MouseButton>,
  mouse_position: Option<(u16, u16)>,
  focused: bool,
  system_bindings: Vec<KeyBinding>,
  bindings: Vec<KeyBinding>,
  action_map_dispatch_enabled: bool,
  raw_key_capture_enabled: bool,
  raw_key_events: VecDeque<RawKeyEvent>,
  raw_mouse_capture_enabled: bool,
  raw_mouse_events: VecDeque<MouseEvent>,
}

impl InputService {
  /// Create an input service with its initial state.
  pub fn new() -> Self {
    let (sender, receiver) = unbounded();
    let (action_sender, action_receiver) = unbounded();
    let (system_sender, system_receiver) = unbounded();

    Self {
      sender,
      receiver,
      action_sender,
      action_receiver,
      system_sender,
      system_receiver,
      pending_system_events: VecDeque::new(),
      observed_system_events: VecDeque::new(),
      key_listener_started: Arc::new(AtomicBool::new(false)),
      system_listener_started: Arc::new(AtomicBool::new(false)),
      held_keys: HashSet::new(),
      blocked_keys: HashSet::new(),
      blocked_page_keys: HashSet::new(),
      blocked_system_keys: HashSet::new(),
      key_transitions: HashMap::new(),
      active_actions: HashMap::new(),
      notifications: Vec::new(),
      frame: 0,
      dispatch_cursor: Cell::new(0),
      pressed_keys: HashSet::new(),
      released_keys: HashSet::new(),
      mouse_held_buttons: HashSet::new(),
      mouse_position: None,
      focused: true,
      system_bindings: Vec::new(),
      bindings: Vec::new(),
      action_map_dispatch_enabled: true,
      raw_key_capture_enabled: false,
      raw_key_events: VecDeque::new(),
      raw_mouse_capture_enabled: false,
      raw_mouse_events: VecDeque::new(),
    }
  }

  /// Start the global keyboard listener once and send observed key changes to the input queue.
  pub fn start_key_listener<E>(&self, async_runtime: &mut AsyncRuntime<E>)
  where
    E: From<KeyEvent>
      + From<InputListenerError>
      + From<tg_service_async::TaskStatusEvent>
      + Send
      + 'static,
  {
    if self.key_listener_started.swap(true, Ordering::SeqCst) {
      return;
    }

    async_runtime.spawn_managed_listener(false, |sender, _stop| {
      thread::spawn(move || {
        let sender_for_callback = sender.clone();
        let callback = move |event: Event| {
          if let Some(key_event) = key_event_from_rdev(event)
            && sender_for_callback.send(E::from(key_event)).is_err()
          {
            // Stop the listener when the application no longer receives keyboard events.
          }
        };
        if let Err(error) = listen(callback) {
          let _ = sender.send(E::from(InputListenerError(format!(
            "Global key listener failed to start: {err:?}",
            err = error
          ))));
        }
      })
    });
  }

  /// Start the terminal listener for key, mouse, resize, and focus events.
  pub fn start_system_listener<E>(&self, async_runtime: &mut AsyncRuntime<E>)
  where
    E: From<SystemEvent>
      + From<CommittedTextEvent>
      + From<InputListenerError>
      + From<tg_service_async::TaskStatusEvent>
      + Send
      + 'static,
  {
    if self.system_listener_started.swap(true, Ordering::SeqCst) {
      return;
    }

    async_runtime.spawn_managed_listener(true, |sender, stop| {
      thread::spawn(move || {
        let poll_interval = Duration::from_millis(50);
        while !stop.load(Ordering::SeqCst) {
          let has_event = match ct_event::poll(poll_interval) {
            Ok(has) => has,
            Err(error) => {
              let _ = sender.send(E::from(InputListenerError(format!(
                "Event poll error: {err}",
                err = error
              ))));
              false
            }
          };
          if has_event {
            if let Ok(ct_event) = ct_event::read() {
              match ct_event {
                CtEvent::Key(key_event) => {
                  if let Some(event) = committed_text_from_crossterm(key_event)
                    && sender.send(E::from(event)).is_err()
                  {
                    break;
                  }
                  if let Some(event) = terminal_key_event_from_crossterm(key_event)
                    && sender.send(E::from(event)).is_err()
                  {
                    break;
                  }
                }
                CtEvent::Paste(text) => {
                  if !text.is_empty() && sender.send(E::from(CommittedTextEvent { text })).is_err()
                  {
                    break;
                  }
                }
                other_event => {
                  if let Some(sys_event) = system_event_from_crossterm(other_event)
                    && sender.send(E::from(sys_event)).is_err()
                  {
                    break;
                  }
                }
              }
            } else {
              // An event may disappear between poll and read; retain the listener after a failed
              // read.
            }
          }
        }
      })
    });
  }

  /// Queue a global key transition for application during input polling.
  pub fn queue_key_event(&self, event: KeyEvent, log: &mut LogService) {
    if self.sender.send(QueuedInput::Key(event)).is_err() {
      log.warn_operation_failed(
        LogSource::Input,
        "queue_key_event",
        "input_channel",
        "channel disconnected",
      );
    }
  }

  /// Queue committed text in the same ordered journal as keys and focus boundaries.
  pub fn queue_committed_text(&self, event: CommittedTextEvent) {
    let _ = self.sender.send(QueuedInput::Text(event));
  }

  /// Queue a terminal event while retaining its UI-consumption order.
  pub fn queue_system_event(&self, event: SystemEvent, log: &mut LogService) {
    if let SystemEvent::Focus(focus) = event {
      let _ = self.sender.send(QueuedInput::Focus(focus));
    }
    if self.system_sender.send(event).is_err() {
      log.warn_operation_failed(
        LogSource::Input,
        "queue_system_event",
        "input_channel",
        "channel disconnected",
      );
    }
  }

  /// Apply queued system events to the current input state.
  pub fn poll_system_events(&mut self) {
    while let Some(event) = self.pop_system_event() {
      self.apply_queued_system_event(&event);
    }
  }

  /// Apply pending terminal size changes before other frame input is routed.
  pub fn poll_resize_events(&mut self, mut on_resize: impl FnMut(u16, u16)) {
    let mut others = VecDeque::new();
    while let Some(event) = self.pop_system_event() {
      match &event {
        SystemEvent::Resize(re) => {
          self.apply_queued_system_event(&event);
          on_resize(re.width, re.height);
        }
        _ => others.push_back(event),
      }
    }
    self.pending_system_events = others;
  }

  /// Drain unobserved system events without consuming the UI's delivery queue.
  pub fn drain_system_event_observations(&mut self, limit: usize) -> Vec<SystemEvent> {
    while self.pending_system_events.len() < limit {
      let Ok(event) = self.system_receiver.try_recv() else {
        break;
      };
      self.pending_system_events.push_back(event);
      self.observed_system_events.push_back(event);
    }
    let count = self.observed_system_events.len().min(limit);
    self.observed_system_events.drain(..count).collect()
  }

  /// Drain system events in order and synthesize pointer hold events for the frame.
  pub fn drain_system_events(&mut self) -> Vec<SystemEvent> {
    self.drain_system_events_limited(128)
  }

  /// Drain at most the frame event budget, leaving excess input queued for later frames.
  pub fn drain_system_events_limited(&mut self, limit: usize) -> Vec<SystemEvent> {
    let mut events = Vec::new();
    let mut active_buttons: HashSet<MouseButton> = HashSet::new();

    while events.len() < limit {
      let Some(event) = self.pop_system_event() else {
        break;
      };
      if let SystemEvent::Mouse(me) = &event
        && let Some(button) = me.button
      {
        match me.kind {
          MouseEventKind::Press | MouseEventKind::Drag => {
            active_buttons.insert(button);
          }
          _ => {}
        }
      }
      if self.focused
        && self.raw_mouse_capture_enabled
        && let SystemEvent::Mouse(mouse) = &event
      {
        self.raw_mouse_events.push_back(*mouse);
      }
      self.apply_queued_system_event(&event);
      events.push(event);
    }

    for button in &self.mouse_held_buttons {
      if events.len() >= limit {
        break;
      }
      if !active_buttons.contains(button)
        && let Some((x, y)) = self.mouse_position
      {
        let hold = MouseEvent {
          kind: MouseEventKind::Hold,
          button: Some(*button),
          scroll: None,
          x,
          y,
        };
        if self.focused && self.raw_mouse_capture_enabled {
          self.raw_mouse_events.push_back(hold);
        }
        events.push(SystemEvent::Mouse(hold));
      }
    }

    events
  }

  fn pop_system_event(&mut self) -> Option<SystemEvent> {
    if let Some(event) = self.pending_system_events.pop_front() {
      return Some(event);
    }
    let event = self.system_receiver.try_recv().ok()?;
    self.observed_system_events.push_back(event);
    Some(event)
  }

  // Keyboard focus is already applied by the ordered journal. Retain pointer cleanup in the
  // system queue without replaying an old loss/gain pair over fresh keyboard input.
  fn apply_queued_system_event(&mut self, event: &SystemEvent) {
    match event {
      SystemEvent::Focus(focus) if !focus.gained => {
        self.mouse_held_buttons.clear();
        self.raw_mouse_events.clear();
      }
      SystemEvent::Focus(_) => {}
      _ => self.apply_system_event(event),
    }
  }

  fn apply_system_event(&mut self, event: &SystemEvent) {
    match event {
      SystemEvent::Focus(focus) => self.apply_focus(*focus),
      SystemEvent::Mouse(me) => {
        self.mouse_position = Some((me.x, me.y));

        if let Some(button) = me.button {
          match me.kind {
            MouseEventKind::Press => {
              self.mouse_held_buttons.insert(button);
            }
            MouseEventKind::Release => {
              self.mouse_held_buttons.remove(&button);
            }
            _ => {}
          }
        }
      }
      _ => {}
    }
  }

  /// Prepare per-frame state and discard submissions or observations belonging to the previous
  /// frame.
  pub fn begin_frame(&mut self) {
    self.pressed_keys.clear();
    self.released_keys.clear();
    self.notifications.clear();
    self.dispatch_cursor.set(0);
    self.frame = self.frame.wrapping_add(1);
  }

  /// Drain queued global keyboard transitions into the current frame key state.
  pub fn poll(&mut self) {
    while let Ok(event) = self.receiver.try_recv() {
      match event {
        QueuedInput::Focus(focus) => self.apply_focus(focus),
        QueuedInput::Text(event) => {
          if self.focused && !event.text.is_empty() {
            self
              .notifications
              .push(InputNotification::Text { text: event.text });
          }
        }
        QueuedInput::Key(event) => {
          if self.focused && self.raw_key_capture_enabled {
            self.raw_key_events.push_back(RawKeyEvent {
              key: event.key,
              display: display_key_token(event.key),
              kind: event.kind,
            });
          }
          self.apply_key_event(event);
        }
      }
    }
    self.advance_held_transitions();
  }

  /// Return the ordered transitions retained for this frame without consuming capture events.
  pub fn notifications(&self) -> &[InputNotification] {
    &self.notifications
  }

  /// Enable raw key capture, clear stale captured events, and return whether the mode changed.
  pub fn enable_raw_key_capture(&mut self) -> bool {
    if self.raw_key_capture_enabled {
      return false;
    }
    self.raw_key_events.clear();
    self.raw_key_capture_enabled = true;
    true
  }

  /// Disable raw key capture and return whether the mode changed.
  pub fn disable_raw_key_capture(&mut self) -> bool {
    if !self.raw_key_capture_enabled {
      return false;
    }
    self.raw_key_capture_enabled = false;
    true
  }

  /// Report whether this input service is raw key capture enabled.
  pub fn is_raw_key_capture_enabled(&self) -> bool {
    self.raw_key_capture_enabled
  }

  /// Report whether this input service is focused.
  pub fn is_focused(&self) -> bool {
    self.focused
  }

  /// Drain the raw keyboard events collected while key capture was enabled.
  pub fn take_raw_key_events(&mut self) -> Vec<RawKeyEvent> {
    self.raw_key_events.drain(..).collect()
  }

  /// Enable raw pointer capture, clear stale captured events, and return whether the mode changed.
  pub fn enable_raw_mouse_capture(&mut self) -> bool {
    if self.raw_mouse_capture_enabled {
      return false;
    }
    self.raw_mouse_events.clear();
    self.raw_mouse_capture_enabled = true;
    true
  }

  /// Disable raw pointer capture and return whether the mode changed.
  pub fn disable_raw_mouse_capture(&mut self) -> bool {
    if !self.raw_mouse_capture_enabled {
      return false;
    }
    self.raw_mouse_capture_enabled = false;
    true
  }

  /// Report whether this input service is raw mouse capture enabled.
  pub fn is_raw_mouse_capture_enabled(&self) -> bool {
    self.raw_mouse_capture_enabled
  }

  /// Drain the raw pointer events collected while mouse capture was enabled.
  pub fn take_raw_mouse_events(&mut self) -> Vec<MouseEvent> {
    self.raw_mouse_events.drain(..).collect()
  }

  /// Report whether the key is currently held down.
  pub fn is_down(&self, key: Key) -> bool {
    self.held_keys.contains(&key)
  }

  /// Report whether the key transitioned to pressed during the current frame.
  pub fn was_pressed(&self, key: Key) -> bool {
    self.pressed_keys.contains(&key)
  }

  /// Report whether the key transitioned to released during the current frame.
  pub fn was_released(&self, key: Key) -> bool {
    self.released_keys.contains(&key)
  }

  /// Return the key state for the addressed object when it is available.
  pub fn key_state(&self, key: Key) -> Option<KeyState> {
    if self.pressed_keys.contains(&key) {
      return Some(KeyState::Pressed);
    }

    if self.released_keys.contains(&key) {
      return Some(KeyState::Released);
    }

    if self.held_keys.contains(&key) {
      return Some(KeyState::Held);
    }

    None
  }

  /// Return the current mouse position.
  pub fn mouse_position(&self) -> Option<(u16, u16)> {
    self.mouse_position
  }

  /// Report whether the addressed object is mouse down.
  pub fn is_mouse_down(&self, button: MouseButton) -> bool {
    self.mouse_held_buttons.contains(&button)
  }

  /// Reset key states, captured events, and queued actions so stale input cannot cross ownership
  /// changes.
  pub fn clear(&mut self) {
    self.notifications.retain(|event| {
      matches!(
        event,
        InputNotification::Key {
          state: KeyState::Released,
          ..
        } | InputNotification::Action {
          event: InputActionEvent {
            state: KeyState::Released,
            ..
          },
          ..
        } | InputNotification::Focus { .. }
      )
    });
    self.dispatch_cursor.set(0);
    while self.action_receiver.try_recv().is_ok() {}
    self.raw_key_events.clear();
    let mut retained = VecDeque::new();
    while let Some(event) = self.pop_system_event() {
      if !matches!(event, SystemEvent::TerminalKey(_)) {
        retained.push_back(event);
      }
    }
    self.pending_system_events = retained;
    self.close_input();
    self.pressed_keys.clear();
    self.released_keys.clear();
  }

  /// Replace the page action bindings used by ordered input dispatch.
  pub fn load_key_bindings(&mut self, mut bindings: Vec<KeyBinding>) {
    for binding in &mut bindings {
      binding.pattern = binding.pattern.normalized();
    }
    if self.bindings == bindings {
      return;
    }
    self.release_actions(Some(false));
    self
      .blocked_page_keys
      .extend(self.held_keys.iter().copied());
    self.bindings = bindings;
  }

  /// Replace host bindings that run before page bindings and remain active when page dispatch is
  /// disabled.
  pub fn load_system_key_bindings(&mut self, mut bindings: Vec<KeyBinding>) {
    for binding in &mut bindings {
      binding.pattern = binding.pattern.normalized();
    }
    if self.system_bindings == bindings {
      return;
    }
    self.release_actions(Some(true));
    self
      .blocked_system_keys
      .extend(self.held_keys.iter().copied());
    self.system_bindings = bindings;
  }

  /// Return the current key bindings.
  pub fn key_bindings(&self) -> &[KeyBinding] {
    &self.bindings
  }

  /// Enable shortcut dispatch and return whether the mode changed.
  pub fn enable_action_map_dispatch(&mut self) -> bool {
    if self.action_map_dispatch_enabled {
      return false;
    }
    self.action_map_dispatch_enabled = true;
    true
  }

  /// Disable shortcut dispatch and return whether the mode changed.
  pub fn disable_action_map_dispatch(&mut self) -> bool {
    if !self.action_map_dispatch_enabled {
      return false;
    }
    self.release_actions(Some(false));
    self
      .blocked_page_keys
      .extend(self.held_keys.iter().copied());
    self.action_map_dispatch_enabled = false;
    true
  }

  /// Report whether this input service is action map dispatch enabled.
  pub fn is_action_map_dispatch_enabled(&self) -> bool {
    self.action_map_dispatch_enabled
  }

  /// Return this frame's ordered, deduplicated action transitions.
  pub fn collect_action_events(&self) -> Vec<InputActionEvent> {
    self
      .notifications
      .iter()
      .filter_map(|notification| match notification {
        InputNotification::Action { event, .. } => Some(event.clone()),
        _ => None,
      })
      .collect()
  }

  fn update_action_states(&mut self) {
    let mut candidates = Vec::new();
    for (system, bindings) in [(true, &self.system_bindings), (false, &self.bindings)] {
      if !system && !self.action_map_dispatch_enabled {
        continue;
      }
      for (registration, binding) in bindings.iter().enumerate() {
        let blocked = if system {
          &self.blocked_system_keys
        } else {
          &self.blocked_page_keys
        };
        let active = |key| self.held_keys.contains(&key) && !blocked.contains(&key);
        let matches = match binding.pattern {
          KeyPattern::Single(key) => active(key),
          KeyPattern::Combo(first, second) => active(first) && active(second),
        };
        if matches {
          candidates.push((
            (system, binding.action.clone()),
            ActiveAction {
              state: KeyState::Pressed,
              frame: self.frame,
              priority: binding.priority,
              combination: matches!(binding.pattern, KeyPattern::Combo(..)),
              registration,
            },
          ));
        }
      }
    }
    candidates.sort_by(|(left, a), (right, b)| Self::action_order(left.0, a, right.0, b));
    let mut matching = HashMap::new();
    for (id, rank) in candidates {
      matching.entry(id).or_insert(rank);
    }
    let mut events = Vec::new();
    for (id, previous) in &self.active_actions {
      if !matching.contains_key(id) {
        events.push((id.clone(), *previous, KeyState::Released));
      }
    }
    for (id, rank) in &mut matching {
      if let Some(previous) = self.active_actions.get(id) {
        rank.state = previous.state;
        rank.frame = previous.frame;
      } else {
        events.push((id.clone(), *rank, KeyState::Pressed));
      }
    }
    self.active_actions = matching;
    events.sort_by(|(a, ar, _), (b, br, _)| Self::action_order(a.0, ar, b.0, br));
    for ((system, action), _, state) in events {
      self.push_action(system, action, state);
    }
  }

  fn action_order(
    system_a: bool,
    a: &ActiveAction,
    system_b: bool,
    b: &ActiveAction,
  ) -> std::cmp::Ordering {
    system_b
      .cmp(&system_a)
      .then(b.priority.cmp(&a.priority))
      .then(b.combination.cmp(&a.combination))
      .then(a.registration.cmp(&b.registration))
  }

  fn push_action(&mut self, system: bool, action: String, state: KeyState) {
    self.notifications.push(InputNotification::Action {
      event: InputActionEvent {
        event_type: InputEventType::Keyboard,
        action,
        state,
      },
      system,
    });
  }

  fn release_actions(&mut self, system: Option<bool>) {
    let mut closing = self
      .active_actions
      .iter()
      .filter(|(id, _)| system.is_none_or(|scope| id.0 == scope))
      .map(|(id, rank)| (id.clone(), *rank))
      .collect::<Vec<_>>();
    closing.sort_by(|(a, ar), (b, br)| Self::action_order(a.0, ar, b.0, br));
    for ((scope, action), _) in closing {
      self.active_actions.remove(&(scope, action.clone()));
      self.push_action(scope, action, KeyState::Released);
    }
  }

  fn close_input(&mut self) {
    self.blocked_keys.extend(self.held_keys.iter().copied());
    let mut keys = self.key_transitions.keys().copied().collect::<Vec<_>>();
    keys.sort_by_key(|key| tg_core_input::key_token(*key));
    for key in keys {
      self.notifications.push(InputNotification::Key {
        key,
        state: KeyState::Released,
      });
    }
    self.key_transitions.clear();
    self.release_actions(None);
    self.held_keys.clear();
  }

  fn apply_focus(&mut self, focus: FocusEvent) {
    if self.focused == focus.gained {
      return;
    }
    self.focused = focus.gained;
    if !focus.gained {
      self.close_input();
      self.raw_key_events.clear();
      self.raw_mouse_events.clear();
      self.pressed_keys.clear();
      self.released_keys.clear();
      self.mouse_held_buttons.clear();
    }
    self.notifications.push(InputNotification::Focus {
      gained: focus.gained,
    });
  }

  fn advance_held_transitions(&mut self) {
    let mut keys = self
      .key_transitions
      .iter()
      .filter(|(_, (state, frame))| *state == KeyState::Pressed && *frame != self.frame)
      .map(|(key, _)| *key)
      .collect::<Vec<_>>();
    keys.sort_by_key(|key| tg_core_input::key_token(*key));
    for key in keys {
      self.key_transitions.get_mut(&key).unwrap().0 = KeyState::Held;
      self.notifications.push(InputNotification::Key {
        key,
        state: KeyState::Held,
      });
    }
    let mut actions = self
      .active_actions
      .iter()
      .filter(|(_, rank)| rank.state == KeyState::Pressed && rank.frame != self.frame)
      .map(|(id, rank)| (id.clone(), *rank))
      .collect::<Vec<_>>();
    actions.sort_by(|(a, ar), (b, br)| Self::action_order(a.0, ar, b.0, br));
    for ((system, action), _) in actions {
      self
        .active_actions
        .get_mut(&(system, action.clone()))
        .unwrap()
        .state = KeyState::Held;
      self.push_action(system, action, KeyState::Held);
    }
  }

  /// Queue all matching host and page action transitions without consuming shared keys.
  pub fn dispatch_action_events(&self, log: &mut LogService) {
    let start = self.dispatch_cursor.get().min(self.notifications.len());
    self.dispatch_cursor.set(self.notifications.len());
    for event in self.notifications[start..]
      .iter()
      .filter_map(|notification| match notification {
        InputNotification::Action { event, .. } => Some(event.clone()),
        _ => None,
      })
    {
      if self.action_sender.send(event).is_err() {
        log.warn_operation_failed(
          LogSource::Input,
          "queue_action_event",
          "input_channel",
          "channel disconnected",
        );
      }
    }
  }

  /// Queue only host actions while page-level dispatch is suspended.
  pub fn dispatch_system_action_events(&self, log: &mut LogService) {
    let start = self.dispatch_cursor.get().min(self.notifications.len());
    self.dispatch_cursor.set(self.notifications.len());
    let events = self.notifications[start..]
      .iter()
      .filter_map(|notification| match notification {
        InputNotification::Action {
          event,
          system: true,
        } => Some(event.clone()),
        _ => None,
      })
      .collect::<Vec<_>>();
    for event in events {
      if self.action_sender.send(event).is_err() {
        log.warn_operation_failed(
          LogSource::Input,
          "queue_action_event",
          "input_channel",
          "channel disconnected",
        );
      }
    }
  }

  /// Pop the next queued action event, or return `None` when the queue is empty.
  pub fn next_action_event(&self) -> Option<InputActionEvent> {
    self.action_receiver.try_recv().ok()
  }

  fn apply_key_event(&mut self, event: KeyEvent) {
    if matches!(event.kind, KeyEventKind::Release) {
      self.blocked_keys.remove(&event.key);
      self.blocked_page_keys.remove(&event.key);
      self.blocked_system_keys.remove(&event.key);
    }
    if !self.focused {
      if event.kind == KeyEventKind::Press {
        self.blocked_keys.insert(event.key);
      }
      return;
    }
    if self.blocked_keys.contains(&event.key) {
      return;
    }
    match event.kind {
      KeyEventKind::Press if self.held_keys.insert(event.key) => {
        self.pressed_keys.insert(event.key);
        self
          .key_transitions
          .insert(event.key, (KeyState::Pressed, self.frame));
        self.notifications.push(InputNotification::Key {
          key: event.key,
          state: KeyState::Pressed,
        });
        self.update_action_states();
      }
      KeyEventKind::Release if self.held_keys.remove(&event.key) => {
        self.released_keys.insert(event.key);
        self.key_transitions.remove(&event.key);
        self.notifications.push(InputNotification::Key {
          key: event.key,
          state: KeyState::Released,
        });
        self.update_action_states();
      }
      _ => {}
    }
  }
}

impl Default for InputService {
  fn default() -> Self {
    Self::new()
  }
}

fn key_from_rdev(key: RdevKey) -> Option<Key> {
  match key {
    RdevKey::Escape => Some(Key::Esc),

    RdevKey::Return => Some(Key::Enter),
    RdevKey::Tab => Some(Key::Tab),
    RdevKey::Backspace => Some(Key::Backspace),
    RdevKey::Space => Some(Key::Space),

    RdevKey::UpArrow => Some(Key::Up),
    RdevKey::DownArrow => Some(Key::Down),
    RdevKey::LeftArrow => Some(Key::Left),
    RdevKey::RightArrow => Some(Key::Right),

    RdevKey::Home => Some(Key::Home),
    RdevKey::End => Some(Key::End),
    RdevKey::PageUp => Some(Key::PageUp),
    RdevKey::PageDown => Some(Key::PageDown),
    RdevKey::Insert => Some(Key::Insert),
    RdevKey::Delete => Some(Key::Delete),

    RdevKey::F1 => Some(Key::Fn(1)),
    RdevKey::F2 => Some(Key::Fn(2)),
    RdevKey::F3 => Some(Key::Fn(3)),
    RdevKey::F4 => Some(Key::Fn(4)),
    RdevKey::F5 => Some(Key::Fn(5)),
    RdevKey::F6 => Some(Key::Fn(6)),
    RdevKey::F7 => Some(Key::Fn(7)),
    RdevKey::F8 => Some(Key::Fn(8)),
    RdevKey::F9 => Some(Key::Fn(9)),
    RdevKey::F10 => Some(Key::Fn(10)),
    RdevKey::F11 => Some(Key::Fn(11)),
    RdevKey::F12 => Some(Key::Fn(12)),

    RdevKey::Num0 => Some(Key::Num(0)),
    RdevKey::Num1 => Some(Key::Num(1)),
    RdevKey::Num2 => Some(Key::Num(2)),
    RdevKey::Num3 => Some(Key::Num(3)),
    RdevKey::Num4 => Some(Key::Num(4)),
    RdevKey::Num5 => Some(Key::Num(5)),
    RdevKey::Num6 => Some(Key::Num(6)),
    RdevKey::Num7 => Some(Key::Num(7)),
    RdevKey::Num8 => Some(Key::Num(8)),
    RdevKey::Num9 => Some(Key::Num(9)),

    RdevKey::KeyA => Some(Key::A),
    RdevKey::KeyB => Some(Key::B),
    RdevKey::KeyC => Some(Key::C),
    RdevKey::KeyD => Some(Key::D),
    RdevKey::KeyE => Some(Key::E),
    RdevKey::KeyF => Some(Key::F),
    RdevKey::KeyG => Some(Key::G),
    RdevKey::KeyH => Some(Key::H),
    RdevKey::KeyI => Some(Key::I),
    RdevKey::KeyJ => Some(Key::J),
    RdevKey::KeyK => Some(Key::K),
    RdevKey::KeyL => Some(Key::L),
    RdevKey::KeyM => Some(Key::M),
    RdevKey::KeyN => Some(Key::N),
    RdevKey::KeyO => Some(Key::O),
    RdevKey::KeyP => Some(Key::P),
    RdevKey::KeyQ => Some(Key::Q),
    RdevKey::KeyR => Some(Key::R),
    RdevKey::KeyS => Some(Key::S),
    RdevKey::KeyT => Some(Key::T),
    RdevKey::KeyU => Some(Key::U),
    RdevKey::KeyV => Some(Key::V),
    RdevKey::KeyW => Some(Key::W),
    RdevKey::KeyX => Some(Key::X),
    RdevKey::KeyY => Some(Key::Y),
    RdevKey::KeyZ => Some(Key::Z),

    RdevKey::ControlLeft => Some(Key::LeftCtrl),
    RdevKey::ControlRight => Some(Key::RightCtrl),

    RdevKey::ShiftLeft => Some(Key::LeftShift),
    RdevKey::ShiftRight => Some(Key::RightShift),

    RdevKey::Alt => Some(Key::LeftAlt),
    RdevKey::AltGr => Some(Key::RightAlt),

    RdevKey::MetaLeft => Some(Key::LeftMeta),
    RdevKey::MetaRight => Some(Key::RightMeta),

    RdevKey::CapsLock => Some(Key::CapsLock),
    RdevKey::NumLock => Some(Key::NumLock),
    RdevKey::ScrollLock => Some(Key::ScrollLock),

    RdevKey::PrintScreen => Some(Key::PrintScreen),
    RdevKey::Pause => Some(Key::Pause),

    RdevKey::BackQuote => Some(Key::BackQuote),
    RdevKey::Minus => Some(Key::Minus),
    RdevKey::Equal => Some(Key::Equal),
    RdevKey::LeftBracket => Some(Key::LeftBracket),
    RdevKey::RightBracket => Some(Key::RightBracket),
    RdevKey::BackSlash => Some(Key::BackSlash),
    RdevKey::SemiColon => Some(Key::Semicolon),
    RdevKey::Quote => Some(Key::Quote),
    RdevKey::Comma => Some(Key::Comma),
    RdevKey::Dot => Some(Key::Dot),
    RdevKey::Slash => Some(Key::Slash),

    RdevKey::Kp0 => Some(Key::Numpad(0)),
    RdevKey::Kp1 => Some(Key::Numpad(1)),
    RdevKey::Kp2 => Some(Key::Numpad(2)),
    RdevKey::Kp3 => Some(Key::Numpad(3)),
    RdevKey::Kp4 => Some(Key::Numpad(4)),
    RdevKey::Kp5 => Some(Key::Numpad(5)),
    RdevKey::Kp6 => Some(Key::Numpad(6)),
    RdevKey::Kp7 => Some(Key::Numpad(7)),
    RdevKey::Kp8 => Some(Key::Numpad(8)),
    RdevKey::Kp9 => Some(Key::Numpad(9)),

    RdevKey::KpPlus => Some(Key::NumpadAdd),
    RdevKey::KpMinus => Some(Key::NumpadSubtract),
    RdevKey::KpMultiply => Some(Key::NumpadMultiply),
    RdevKey::KpDivide => Some(Key::NumpadDivide),
    RdevKey::KpReturn => Some(Key::NumpadEnter),
    RdevKey::KpDelete => Some(Key::NumpadDelete),

    RdevKey::Unknown(code) => Some(Key::Unknown(code)),
    _ => None,
  }
}

fn key_event_from_rdev(event: Event) -> Option<KeyEvent> {
  match event.event_type {
    EventType::KeyPress(key) => key_from_rdev(key).map(|key| KeyEvent {
      key,
      kind: KeyEventKind::Press,
    }),
    EventType::KeyRelease(key) => key_from_rdev(key).map(|key| KeyEvent {
      key,
      kind: KeyEventKind::Release,
    }),
    _ => None,
  }
}

fn committed_text_from_crossterm(event: CtKeyEvent) -> Option<CommittedTextEvent> {
  if event.kind == CtKeyEventKind::Release
    || event.modifiers.intersects(
      CtKeyModifiers::CONTROL
        | CtKeyModifiers::ALT
        | CtKeyModifiers::SUPER
        | CtKeyModifiers::HYPER
        | CtKeyModifiers::META,
    )
  {
    return None;
  }
  match event.code {
    CtKeyCode::Char(character) if !character.is_control() => Some(CommittedTextEvent {
      text: character.to_string(),
    }),
    _ => None,
  }
}

fn terminal_key_event_from_crossterm(event: CtKeyEvent) -> Option<SystemEvent> {
  if event.kind == CtKeyEventKind::Release {
    return None;
  }

  let ctrl = event.modifiers.contains(CtKeyModifiers::CONTROL);
  let shift = event.modifiers.contains(CtKeyModifiers::SHIFT);
  let rejected =
    CtKeyModifiers::ALT | CtKeyModifiers::SUPER | CtKeyModifiers::HYPER | CtKeyModifiers::META;
  if event.modifiers.intersects(rejected) {
    return None;
  }
  let allowed_modifiers = match event.code {
    CtKeyCode::Char(ch) if ctrl => "acxv".contains(ch.to_ascii_lowercase()) && !shift,
    CtKeyCode::Char(_) => !ctrl,
    CtKeyCode::Enter => !shift,
    CtKeyCode::Left | CtKeyCode::Right => true,
    CtKeyCode::Up | CtKeyCode::Down | CtKeyCode::Home | CtKeyCode::End => !ctrl,
    _ => !ctrl && !shift,
  };
  if !allowed_modifiers {
    return None;
  }

  let code = match event.code {
    CtKeyCode::Char(ch) if !ch.is_control() => TerminalKeyCode::Char(ch),
    CtKeyCode::Enter => TerminalKeyCode::Enter,
    CtKeyCode::Esc => TerminalKeyCode::Esc,
    CtKeyCode::Backspace => TerminalKeyCode::Backspace,
    CtKeyCode::Delete => TerminalKeyCode::Delete,
    CtKeyCode::Left => TerminalKeyCode::Left,
    CtKeyCode::Right => TerminalKeyCode::Right,
    CtKeyCode::Up => TerminalKeyCode::Up,
    CtKeyCode::Down => TerminalKeyCode::Down,
    CtKeyCode::Home => TerminalKeyCode::Home,
    CtKeyCode::End => TerminalKeyCode::End,
    _ => return None,
  };

  Some(SystemEvent::TerminalKey(TerminalKeyEvent {
    code,
    ctrl,
    shift,
  }))
}

fn system_event_from_crossterm(event: CtEvent) -> Option<SystemEvent> {
  match event {
    CtEvent::Resize(width, height) => Some(SystemEvent::Resize(ResizeEvent { width, height })),
    CtEvent::FocusGained => Some(SystemEvent::Focus(FocusEvent { gained: true })),
    CtEvent::FocusLost => Some(SystemEvent::Focus(FocusEvent { gained: false })),
    CtEvent::Mouse(me) => Some(SystemEvent::Mouse(mouse_event_from_crossterm(me))),
    _ => None,
  }
}

fn mouse_event_from_crossterm(me: CtMouseEvent) -> MouseEvent {
  let (kind, button, scroll) = match me.kind {
    CtMouseEventKind::Down(btn) => (
      MouseEventKind::Press,
      Some(mouse_button_from_crossterm(btn)),
      None,
    ),
    CtMouseEventKind::Up(btn) => (
      MouseEventKind::Release,
      Some(mouse_button_from_crossterm(btn)),
      None,
    ),
    CtMouseEventKind::Drag(btn) => (
      MouseEventKind::Drag,
      Some(mouse_button_from_crossterm(btn)),
      None,
    ),
    CtMouseEventKind::Moved => (MouseEventKind::Move, None, None),
    CtMouseEventKind::ScrollDown => (MouseEventKind::Scroll, None, Some(ScrollDirection::Down)),
    CtMouseEventKind::ScrollUp => (MouseEventKind::Scroll, None, Some(ScrollDirection::Up)),
    CtMouseEventKind::ScrollLeft => (MouseEventKind::Scroll, None, Some(ScrollDirection::Left)),
    CtMouseEventKind::ScrollRight => (MouseEventKind::Scroll, None, Some(ScrollDirection::Right)),
  };

  MouseEvent {
    kind,
    button,
    scroll,
    x: me.column,
    y: me.row,
  }
}

fn mouse_button_from_crossterm(button: crossterm::event::MouseButton) -> MouseButton {
  match button {
    crossterm::event::MouseButton::Left => MouseButton::Left,
    crossterm::event::MouseButton::Middle => MouseButton::Middle,
    crossterm::event::MouseButton::Right => MouseButton::Right,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tg_core_input::{ActionMapEntry, translate_action_map};

  fn key(code: CtKeyCode) -> CtKeyEvent {
    CtKeyEvent::new(code, CtKeyModifiers::NONE)
  }

  fn terminal_code(event: CtKeyEvent) -> Option<TerminalKeyCode> {
    match terminal_key_event_from_crossterm(event) {
      Some(SystemEvent::TerminalKey(event)) => Some(event.code),
      _ => None,
    }
  }

  fn terminal_event(event: CtKeyEvent) -> Option<TerminalKeyEvent> {
    match terminal_key_event_from_crossterm(event) {
      Some(SystemEvent::TerminalKey(event)) => Some(event),
      _ => None,
    }
  }

  #[test]
  fn terminal_text_accepts_committed_characters_and_repeats_without_shortcuts() {
    let event = |code, modifiers, kind| CtKeyEvent::new_with_kind(code, modifiers, kind);
    for (character, modifiers) in [
      ('a', CtKeyModifiers::NONE),
      ('中', CtKeyModifiers::NONE),
      ('A', CtKeyModifiers::SHIFT),
    ] {
      for kind in [CtKeyEventKind::Press, CtKeyEventKind::Repeat] {
        assert_eq!(
          committed_text_from_crossterm(event(CtKeyCode::Char(character), modifiers, kind)),
          Some(CommittedTextEvent {
            text: character.to_string()
          })
        );
      }
    }
    for (code, modifiers, kind) in [
      (
        CtKeyCode::Char('a'),
        CtKeyModifiers::NONE,
        CtKeyEventKind::Release,
      ),
      (
        CtKeyCode::Char('c'),
        CtKeyModifiers::CONTROL,
        CtKeyEventKind::Press,
      ),
      (
        CtKeyCode::Char('a'),
        CtKeyModifiers::ALT,
        CtKeyEventKind::Press,
      ),
      (
        CtKeyCode::Enter,
        CtKeyModifiers::NONE,
        CtKeyEventKind::Press,
      ),
      (
        CtKeyCode::Backspace,
        CtKeyModifiers::NONE,
        CtKeyEventKind::Press,
      ),
    ] {
      assert!(committed_text_from_crossterm(event(code, modifiers, kind)).is_none());
    }
  }

  #[test]
  fn committed_text_keeps_focus_order_and_does_not_consume_widget_events() {
    let mut input = InputService::new();
    let mut log = LogService::new();
    input.begin_frame();
    input.queue_committed_text(CommittedTextEvent {
      text: "first".into(),
    });
    input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: false }), &mut log);
    input.queue_committed_text(CommittedTextEvent {
      text: "unfocused".into(),
    });
    input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: true }), &mut log);
    input.queue_committed_text(CommittedTextEvent {
      text: "paste\n中".into(),
    });
    input.queue_committed_text(CommittedTextEvent {
      text: String::new(),
    });
    input.poll();
    assert_eq!(
      input.notifications(),
      [
        InputNotification::Text {
          text: "first".into()
        },
        InputNotification::Focus { gained: false },
        InputNotification::Focus { gained: true },
        InputNotification::Text {
          text: "paste\n中".into()
        },
      ]
    );
    input.clear();
    assert!(
      input
        .notifications()
        .iter()
        .all(|event| matches!(event, InputNotification::Focus { .. }))
    );
  }

  #[test]
  fn raw_key_capture_runs_alongside_action_map() {
    let mut input = InputService::new();
    input.load_key_bindings(vec![KeyBinding {
      pattern: KeyPattern::Single(Key::A),
      action: "test.a".to_string(),
      priority: 0,
    }]);
    assert!(input.enable_raw_key_capture());
    assert!(!input.enable_raw_key_capture());

    input
      .sender
      .send(QueuedInput::Key(KeyEvent {
        key: Key::A,
        kind: KeyEventKind::Press,
      }))
      .unwrap();
    input.poll();

    assert_eq!(
      input.take_raw_key_events(),
      vec![RawKeyEvent {
        key: Key::A,
        display: "A".to_string(),
        kind: KeyEventKind::Press,
      }]
    );
    assert_eq!(input.collect_action_events()[0].action, "test.a");
    assert_eq!(input.collect_action_events()[0].state, KeyState::Pressed);
  }

  #[test]
  fn raw_key_capture_uses_the_shared_modifier_display() {
    let mut input = InputService::new();
    input.enable_raw_key_capture();
    for key in [Key::LeftCtrl, Key::RightCtrl] {
      input
        .sender
        .send(QueuedInput::Key(KeyEvent {
          key,
          kind: KeyEventKind::Press,
        }))
        .unwrap();
    }
    input.poll();

    let events = input.take_raw_key_events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].key, Key::LeftCtrl);
    assert_eq!(events[0].display, display_key_token(Key::LeftCtrl));
    assert_eq!(events[1].key, Key::RightCtrl);
    assert_eq!(events[1].display, display_key_token(Key::RightCtrl));
    assert_ne!(events[0].display, events[1].display);
  }

  #[test]
  fn raw_key_capture_disable_preserves_events_and_enable_starts_clean() {
    let mut input = InputService::new();
    input.enable_raw_key_capture();
    input
      .sender
      .send(QueuedInput::Key(KeyEvent {
        key: Key::Left,
        kind: KeyEventKind::Press,
      }))
      .unwrap();
    input.poll();
    assert!(input.disable_raw_key_capture());
    assert!(!input.disable_raw_key_capture());
    assert_eq!(input.take_raw_key_events()[0].display, "←");

    input.enable_raw_key_capture();
    input
      .sender
      .send(QueuedInput::Key(KeyEvent {
        key: Key::Left,
        kind: KeyEventKind::Release,
      }))
      .unwrap();
    input.poll();
    assert_eq!(input.take_raw_key_events()[0].kind, KeyEventKind::Release);

    input
      .sender
      .send(QueuedInput::Key(KeyEvent {
        key: Key::B,
        kind: KeyEventKind::Press,
      }))
      .unwrap();
    input.poll();
    input.disable_raw_key_capture();
    input.enable_raw_key_capture();
    assert!(input.take_raw_key_events().is_empty());
  }

  #[test]
  fn all_matching_actions_use_scope_priority_pattern_and_registration_order() {
    let mut input = InputService::new();
    input.load_system_key_bindings(vec![KeyBinding {
      pattern: KeyPattern::Single(Key::A),
      action: "host".into(),
      priority: 0,
    }]);
    input.load_key_bindings(vec![
      KeyBinding {
        pattern: KeyPattern::Single(Key::A),
        action: "early".into(),
        priority: 0,
      },
      KeyBinding {
        pattern: KeyPattern::Combo(Key::LeftCtrl, Key::A),
        action: "combo".into(),
        priority: 0,
      },
      KeyBinding {
        pattern: KeyPattern::Single(Key::A),
        action: "high".into(),
        priority: 10,
      },
      KeyBinding {
        pattern: KeyPattern::Single(Key::A),
        action: "late".into(),
        priority: 0,
      },
    ]);
    input.apply_key_event(KeyEvent {
      key: Key::LeftCtrl,
      kind: KeyEventKind::Press,
    });
    input.apply_key_event(KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Press,
    });
    assert_eq!(
      input
        .collect_action_events()
        .iter()
        .map(|event| event.action.as_str())
        .collect::<Vec<_>>(),
      ["host", "high", "combo", "early", "late"]
    );
    input.begin_frame();
    input.poll();
    assert_eq!(input.collect_action_events().len(), 5);
    assert!(
      input
        .collect_action_events()
        .iter()
        .all(|event| event.state == KeyState::Held)
    );
    input.begin_frame();
    input.poll();
    assert!(input.collect_action_events().is_empty());
    input.apply_key_event(KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Release,
    });
    assert_eq!(input.collect_action_events().len(), 5);
    assert!(
      input
        .collect_action_events()
        .iter()
        .all(|event| event.state == KeyState::Released)
    );
  }

  #[test]
  fn alternative_bindings_release_only_when_the_last_binding_ends() {
    let mut input = InputService::new();
    input.load_key_bindings(vec![
      KeyBinding {
        pattern: KeyPattern::Single(Key::A),
        action: "move".into(),
        priority: 0,
      },
      KeyBinding {
        pattern: KeyPattern::Single(Key::B),
        action: "move".into(),
        priority: 0,
      },
    ]);
    for key in [Key::A, Key::B] {
      input.apply_key_event(KeyEvent {
        key,
        kind: KeyEventKind::Press,
      });
    }
    assert_eq!(input.collect_action_events().len(), 1);
    input.begin_frame();
    input.poll();
    input.begin_frame();
    input.apply_key_event(KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Release,
    });
    assert!(input.collect_action_events().is_empty());
    input.apply_key_event(KeyEvent {
      key: Key::B,
      kind: KeyEventKind::Release,
    });
    assert_eq!(input.collect_action_events()[0].state, KeyState::Released);
  }

  #[test]
  fn fast_taps_and_autorepeat_preserve_edges_and_capture_is_independent() {
    let mut input = InputService::new();
    input.enable_raw_key_capture();
    input.load_key_bindings(vec![KeyBinding {
      pattern: KeyPattern::Single(Key::A),
      action: "a".into(),
      priority: 0,
    }]);
    let mut log = LogService::new();
    for kind in [
      KeyEventKind::Press,
      KeyEventKind::Press,
      KeyEventKind::Release,
      KeyEventKind::Press,
      KeyEventKind::Release,
    ] {
      input.queue_key_event(KeyEvent { key: Key::A, kind }, &mut log);
    }
    input.poll();
    let events = input.collect_action_events();
    assert_eq!(
      events.iter().map(|event| event.state).collect::<Vec<_>>(),
      [
        KeyState::Pressed,
        KeyState::Released,
        KeyState::Pressed,
        KeyState::Released
      ]
    );
    assert_eq!(
      input
        .notifications()
        .iter()
        .filter(|event| matches!(event, InputNotification::Key { .. }))
        .count(),
      4
    );
    assert_eq!(input.take_raw_key_events().len(), 5);
  }

  #[test]
  fn focus_and_binding_changes_close_input_without_inheriting_held_keys() {
    let mut input = InputService::new();
    input.load_key_bindings(vec![KeyBinding {
      pattern: KeyPattern::Single(Key::A),
      action: "old".into(),
      priority: 0,
    }]);
    let mut log = LogService::new();
    input.queue_key_event(
      KeyEvent {
        key: Key::A,
        kind: KeyEventKind::Press,
      },
      &mut log,
    );
    input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: false }), &mut log);
    input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: true }), &mut log);
    input.queue_key_event(
      KeyEvent {
        key: Key::A,
        kind: KeyEventKind::Press,
      },
      &mut log,
    );
    input.poll();
    assert_eq!(
      input
        .collect_action_events()
        .iter()
        .map(|event| event.state)
        .collect::<Vec<_>>(),
      [KeyState::Pressed, KeyState::Released]
    );
    input.begin_frame();
    input.queue_key_event(
      KeyEvent {
        key: Key::A,
        kind: KeyEventKind::Release,
      },
      &mut log,
    );
    input.queue_key_event(
      KeyEvent {
        key: Key::A,
        kind: KeyEventKind::Press,
      },
      &mut log,
    );
    input.poll();
    assert_eq!(input.collect_action_events()[0].state, KeyState::Pressed);
    input.begin_frame();
    input.load_key_bindings(vec![KeyBinding {
      pattern: KeyPattern::Single(Key::A),
      action: "new".into(),
      priority: 0,
    }]);
    input.poll();
    assert_eq!(input.collect_action_events()[0].action, "old");
    assert_eq!(input.collect_action_events()[0].state, KeyState::Released);
    assert_eq!(input.collect_action_events().len(), 1);
  }

  #[test]
  fn unchanged_noncanonical_combinations_do_not_reset_active_input() {
    let mut input = InputService::new();
    let bindings = vec![KeyBinding {
      action: "combo".into(),
      pattern: KeyPattern::Combo(Key::Z, Key::A),
      priority: 0,
    }];
    input.load_key_bindings(bindings.clone());
    for key in [Key::A, Key::Z] {
      input.apply_key_event(KeyEvent {
        key,
        kind: KeyEventKind::Press,
      });
    }
    input.begin_frame();
    input.load_key_bindings(bindings);
    input.poll();
    assert_eq!(input.collect_action_events().len(), 1);
    assert_eq!(input.collect_action_events()[0].state, KeyState::Held);
  }

  #[test]
  fn actual_single_match_is_not_ranked_as_an_inactive_alternative_combo() {
    let mut input = InputService::new();
    input.load_key_bindings(vec![
      KeyBinding {
        action: "first".into(),
        pattern: KeyPattern::Single(Key::A),
        priority: 0,
      },
      KeyBinding {
        action: "mixed".into(),
        pattern: KeyPattern::Single(Key::A),
        priority: 0,
      },
      KeyBinding {
        action: "mixed".into(),
        pattern: KeyPattern::Combo(Key::B, Key::C),
        priority: 0,
      },
    ]);
    input.apply_key_event(KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Press,
    });
    assert_eq!(
      input
        .collect_action_events()
        .iter()
        .map(|event| event.action.as_str())
        .collect::<Vec<_>>(),
      ["first", "mixed"]
    );
  }

  #[test]
  fn system_queue_observation_does_not_replay_focus_or_close_fresh_input() {
    let mut input = InputService::new();
    let mut log = LogService::new();
    input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: false }), &mut log);
    input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: true }), &mut log);
    input.queue_key_event(
      KeyEvent {
        key: Key::A,
        kind: KeyEventKind::Press,
      },
      &mut log,
    );
    input.poll();
    let notifications = input.notifications().to_vec();
    assert_eq!(input.drain_system_events().len(), 2);
    assert!(input.is_down(Key::A));
    assert_eq!(input.notifications(), notifications);
  }

  #[test]
  fn changed_host_mapping_waits_for_fresh_press_and_clear_discards_ui_queue() {
    let mut input = InputService::new();
    let mut log = LogService::new();
    let binding = |name: &str| KeyBinding {
      action: name.into(),
      pattern: KeyPattern::Single(Key::A),
      priority: 0,
    };
    input.load_system_key_bindings(vec![binding("old")]);
    input.apply_key_event(KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Press,
    });
    input.dispatch_action_events(&mut log);
    input.clear();
    assert!(input.next_action_event().is_none());
    input.begin_frame();
    input.apply_key_event(KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Release,
    });
    input.apply_key_event(KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Press,
    });
    input.begin_frame();
    input.load_system_key_bindings(vec![binding("new")]);
    input.apply_key_event(KeyEvent {
      key: Key::B,
      kind: KeyEventKind::Press,
    });
    assert_eq!(
      input
        .collect_action_events()
        .iter()
        .map(|event| (event.action.as_str(), event.state))
        .collect::<Vec<_>>(),
      [("old", KeyState::Released)]
    );
  }

  #[test]
  fn action_map_still_reports_pressed_held_and_released() {
    let mut input = InputService::new();
    input.load_key_bindings(vec![KeyBinding {
      pattern: KeyPattern::Single(Key::A),
      action: "test.a".to_string(),
      priority: 0,
    }]);
    input.apply_key_event(KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Press,
    });
    assert_eq!(input.collect_action_events()[0].state, KeyState::Pressed);
    input.begin_frame();
    input.poll();
    assert_eq!(input.collect_action_events()[0].state, KeyState::Held);
    input.begin_frame();
    input.apply_key_event(KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Release,
    });
    assert_eq!(input.collect_action_events()[0].state, KeyState::Released);
  }

  #[test]
  fn left_and_right_modifier_bindings_route_independently() {
    let bindings = translate_action_map(&[
      ActionMapEntry {
        action: "left_modifier".into(),
        description: String::new(),
        keys: vec![vec!["left_ctrl".into()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "right_modifier".into(),
        description: String::new(),
        keys: vec![vec!["right_ctrl".into()]],
        priority: 0,
      },
    ])
    .unwrap();
    let mut input = InputService::new();
    input.load_key_bindings(bindings);

    input
      .sender
      .send(QueuedInput::Key(KeyEvent {
        key: Key::LeftCtrl,
        kind: KeyEventKind::Press,
      }))
      .unwrap();
    input.poll();
    assert_eq!(
      input.collect_action_events(),
      vec![InputActionEvent {
        event_type: InputEventType::Keyboard,
        action: "left_modifier".into(),
        state: KeyState::Pressed,
      }]
    );

    input.begin_frame();
    input
      .sender
      .send(QueuedInput::Key(KeyEvent {
        key: Key::RightCtrl,
        kind: KeyEventKind::Press,
      }))
      .unwrap();
    input.poll();
    assert_eq!(
      input.collect_action_events(),
      vec![
        InputActionEvent {
          event_type: InputEventType::Keyboard,
          action: "right_modifier".into(),
          state: KeyState::Pressed,
        },
        InputActionEvent {
          event_type: InputEventType::Keyboard,
          action: "left_modifier".into(),
          state: KeyState::Held,
        },
      ]
    );

    input.begin_frame();
    input
      .sender
      .send(QueuedInput::Key(KeyEvent {
        key: Key::LeftCtrl,
        kind: KeyEventKind::Release,
      }))
      .unwrap();
    input.poll();
    assert_eq!(
      input.collect_action_events(),
      vec![
        InputActionEvent {
          event_type: InputEventType::Keyboard,
          action: "left_modifier".into(),
          state: KeyState::Released,
        },
        InputActionEvent {
          event_type: InputEventType::Keyboard,
          action: "right_modifier".into(),
          state: KeyState::Held,
        },
      ]
    );
  }

  #[test]
  fn rdev_modifier_events_keep_the_reported_side() {
    let cases = [
      (RdevKey::ControlLeft, Key::LeftCtrl),
      (RdevKey::ControlRight, Key::RightCtrl),
      (RdevKey::ShiftLeft, Key::LeftShift),
      (RdevKey::ShiftRight, Key::RightShift),
      (RdevKey::Alt, Key::LeftAlt),
      (RdevKey::AltGr, Key::RightAlt),
      (RdevKey::MetaLeft, Key::LeftMeta),
      (RdevKey::MetaRight, Key::RightMeta),
    ];
    for (source, expected) in cases {
      assert_eq!(key_from_rdev(source), Some(expected));
    }
  }

  #[test]
  fn left_and_right_modifier_combinations_route_to_distinct_actions() {
    fn send(input: &mut InputService, key: Key, kind: KeyEventKind) {
      input
        .sender
        .send(QueuedInput::Key(KeyEvent { key, kind }))
        .unwrap();
      input.poll();
    }

    let bindings = translate_action_map(&[
      ActionMapEntry {
        action: "left_combo".into(),
        description: String::new(),
        keys: vec![vec!["left_ctrl".into(), "x".into()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "right_combo".into(),
        description: String::new(),
        keys: vec![vec!["right_ctrl".into(), "x".into()]],
        priority: 0,
      },
    ])
    .unwrap();
    let mut input = InputService::new();
    input.load_key_bindings(bindings);

    send(&mut input, Key::LeftCtrl, KeyEventKind::Press);
    assert!(input.collect_action_events().is_empty());
    input.begin_frame();
    send(&mut input, Key::X, KeyEventKind::Press);
    let events = input.collect_action_events();
    assert_eq!(events[0].action, "left_combo");
    assert_eq!(events[0].state, KeyState::Pressed);
    input.begin_frame();
    send(&mut input, Key::X, KeyEventKind::Release);
    let events = input.collect_action_events();
    assert_eq!(events[0].action, "left_combo");
    assert_eq!(events[0].state, KeyState::Released);
    input.begin_frame();
    send(&mut input, Key::LeftCtrl, KeyEventKind::Release);
    assert!(input.collect_action_events().is_empty());

    input.begin_frame();
    send(&mut input, Key::RightCtrl, KeyEventKind::Press);
    assert!(input.collect_action_events().is_empty());
    input.begin_frame();
    send(&mut input, Key::X, KeyEventKind::Press);
    let events = input.collect_action_events();
    assert_eq!(events[0].action, "right_combo");
    assert_eq!(events[0].state, KeyState::Pressed);
  }

  #[test]
  fn action_map_dispatch_can_be_disabled_without_affecting_raw_capture() {
    let mut input = InputService::new();
    input.load_key_bindings(vec![KeyBinding {
      pattern: KeyPattern::Single(Key::A),
      action: "test.a".to_string(),
      priority: 0,
    }]);
    assert!(input.is_action_map_dispatch_enabled());
    assert!(input.disable_action_map_dispatch());
    assert!(!input.disable_action_map_dispatch());
    assert!(input.enable_raw_key_capture());

    input
      .sender
      .send(QueuedInput::Key(KeyEvent {
        key: Key::A,
        kind: KeyEventKind::Press,
      }))
      .unwrap();
    input.poll();
    input.dispatch_action_events(&mut LogService::new());

    assert!(input.next_action_event().is_none());
    assert_eq!(input.take_raw_key_events()[0].display, "A");
    assert!(input.collect_action_events().is_empty());

    input.load_system_key_bindings(vec![KeyBinding {
      pattern: KeyPattern::Single(Key::Fn(4)),
      action: "host_key.force_stop".to_string(),
      priority: 0,
    }]);
    input.apply_key_event(KeyEvent {
      key: Key::Fn(4),
      kind: KeyEventKind::Press,
    });
    input.dispatch_action_events(&mut LogService::new());
    assert_eq!(
      input.next_action_event().unwrap().action,
      "host_key.force_stop"
    );
    input.apply_key_event(KeyEvent {
      key: Key::Fn(4),
      kind: KeyEventKind::Release,
    });
    input.dispatch_action_events(&mut LogService::new());
    assert_eq!(
      input.next_action_event().unwrap().action,
      "host_key.force_stop"
    );
    input.begin_frame();

    assert!(input.enable_action_map_dispatch());
    assert!(!input.enable_action_map_dispatch());
    input.dispatch_action_events(&mut LogService::new());
    assert!(input.next_action_event().is_none());
    input.apply_key_event(KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Release,
    });
    input.apply_key_event(KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Press,
    });
    input.dispatch_action_events(&mut LogService::new());
    assert_eq!(input.next_action_event().unwrap().action, "test.a");
  }

  #[test]
  fn mouse_queries_track_position_buttons_and_focus_loss() {
    let mut input = InputService::new();
    assert_eq!(input.mouse_position(), None);
    assert!(!input.is_mouse_down(MouseButton::Middle));
    input.apply_system_event(&SystemEvent::Mouse(MouseEvent {
      kind: MouseEventKind::Press,
      button: Some(MouseButton::Middle),
      scroll: None,
      x: 7,
      y: 9,
    }));
    assert_eq!(input.mouse_position(), Some((7, 9)));
    assert!(input.is_mouse_down(MouseButton::Middle));
    input.apply_system_event(&SystemEvent::Focus(FocusEvent { gained: false }));
    assert!(!input.is_mouse_down(MouseButton::Middle));
    assert_eq!(input.mouse_position(), Some((7, 9)));
  }

  #[test]
  fn terminal_key_event_from_crossterm_maps_supported_keys() {
    let cases = [
      (CtKeyCode::Char('a'), TerminalKeyCode::Char('a')),
      (CtKeyCode::Char('我'), TerminalKeyCode::Char('我')),
      (CtKeyCode::Enter, TerminalKeyCode::Enter),
      (CtKeyCode::Esc, TerminalKeyCode::Esc),
      (CtKeyCode::Backspace, TerminalKeyCode::Backspace),
      (CtKeyCode::Delete, TerminalKeyCode::Delete),
      (CtKeyCode::Left, TerminalKeyCode::Left),
      (CtKeyCode::Right, TerminalKeyCode::Right),
      (CtKeyCode::Up, TerminalKeyCode::Up),
      (CtKeyCode::Down, TerminalKeyCode::Down),
      (CtKeyCode::Home, TerminalKeyCode::Home),
      (CtKeyCode::End, TerminalKeyCode::End),
    ];

    for (input, expected) in cases {
      assert_eq!(terminal_code(key(input)), Some(expected));
    }
    assert_eq!(terminal_code(key(CtKeyCode::F(1))), None);
    assert_eq!(terminal_code(key(CtKeyCode::Tab)), None);
  }

  #[test]
  fn terminal_key_event_from_crossterm_filters_kind_and_modifiers() {
    assert_eq!(
      terminal_code(CtKeyEvent::new_with_kind(
        CtKeyCode::Char('a'),
        CtKeyModifiers::NONE,
        CtKeyEventKind::Repeat,
      )),
      Some(TerminalKeyCode::Char('a'))
    );
    assert_eq!(
      terminal_code(CtKeyEvent::new_with_kind(
        CtKeyCode::Char('a'),
        CtKeyModifiers::NONE,
        CtKeyEventKind::Release,
      )),
      None
    );
    assert_eq!(
      terminal_code(CtKeyEvent::new(CtKeyCode::Char('A'), CtKeyModifiers::SHIFT,)),
      Some(TerminalKeyCode::Char('A'))
    );
    assert_eq!(
      terminal_event(CtKeyEvent::new(CtKeyCode::Enter, CtKeyModifiers::CONTROL)),
      Some(TerminalKeyEvent {
        code: TerminalKeyCode::Enter,
        ctrl: true,
        shift: false,
      })
    );
    assert_eq!(
      terminal_event(CtKeyEvent::new(
        CtKeyCode::Char('a'),
        CtKeyModifiers::CONTROL,
      )),
      Some(TerminalKeyEvent {
        code: TerminalKeyCode::Char('a'),
        ctrl: true,
        shift: false,
      })
    );
    assert_eq!(
      terminal_code(CtKeyEvent::new(
        CtKeyCode::Left,
        CtKeyModifiers::CONTROL | CtKeyModifiers::SHIFT,
      )),
      Some(TerminalKeyCode::Left)
    );
    assert_eq!(
      terminal_code(CtKeyEvent::new(
        CtKeyCode::Char('z'),
        CtKeyModifiers::CONTROL,
      )),
      None
    );
    for modifiers in [
      CtKeyModifiers::ALT,
      CtKeyModifiers::SUPER,
      CtKeyModifiers::HYPER,
      CtKeyModifiers::META,
    ] {
      assert_eq!(
        terminal_code(CtKeyEvent::new(CtKeyCode::Char('a'), modifiers)),
        None
      );
    }
  }

  #[test]
  fn terminal_key_survives_resize_poll() {
    let mut input = InputService::new();
    let a = SystemEvent::TerminalKey(TerminalKeyEvent {
      code: TerminalKeyCode::Char('a'),
      ctrl: false,
      shift: false,
    });
    let b = SystemEvent::TerminalKey(TerminalKeyEvent {
      code: TerminalKeyCode::Char('b'),
      ctrl: false,
      shift: false,
    });
    input.system_sender.send(a).unwrap();
    input
      .system_sender
      .send(SystemEvent::Resize(ResizeEvent {
        width: 100,
        height: 30,
      }))
      .unwrap();
    input.system_sender.send(b).unwrap();

    let mut resize = None;
    input.poll_resize_events(|width, height| resize = Some((width, height)));

    assert_eq!(resize, Some((100, 30)));
    assert_eq!(input.drain_system_events(), vec![a, b]);
  }

  #[test]
  fn ownership_cleanup_drops_terminal_keys_but_keeps_resize_and_focus_observations() {
    let mut input = InputService::new();
    let mut log = LogService::new();
    input.queue_system_event(
      SystemEvent::TerminalKey(TerminalKeyEvent {
        code: TerminalKeyCode::Enter,
        ctrl: false,
        shift: false,
      }),
      &mut log,
    );
    let resize = SystemEvent::Resize(ResizeEvent {
      width: 100,
      height: 30,
    });
    input.queue_system_event(resize, &mut log);
    let focus = SystemEvent::Focus(FocusEvent { gained: false });
    input.queue_system_event(focus, &mut log);
    input.poll();
    input.clear();
    assert_eq!(input.drain_system_events(), [resize, focus]);
    assert!(
      input
        .notifications()
        .iter()
        .any(|event| matches!(event, InputNotification::Focus { gained: false }))
    );
  }

  #[test]
  fn system_event_drain_keeps_input_beyond_the_frame_limit() {
    let mut input = InputService::new();
    for _ in 0..130 {
      input
        .system_sender
        .send(SystemEvent::TerminalKey(TerminalKeyEvent {
          code: TerminalKeyCode::Char('a'),
          ctrl: false,
          shift: false,
        }))
        .unwrap();
    }

    assert_eq!(input.drain_system_events().len(), 128);
    assert_eq!(input.drain_system_events().len(), 2);
  }

  #[test]
  fn system_event_observation_preserves_events_for_the_normal_consumer() {
    let mut input = InputService::new();
    let focus = SystemEvent::Focus(FocusEvent { gained: false });
    let key = SystemEvent::TerminalKey(TerminalKeyEvent {
      code: TerminalKeyCode::Char('a'),
      ctrl: false,
      shift: false,
    });
    input.system_sender.send(focus).unwrap();
    input.system_sender.send(key).unwrap();

    assert_eq!(input.drain_system_event_observations(128), vec![focus, key]);
    assert!(input.drain_system_event_observations(128).is_empty());
    assert_eq!(input.drain_system_events(), vec![focus, key]);
  }

  #[test]
  fn raw_mouse_capture_runs_alongside_system_events() {
    let mut input = InputService::new();
    assert!(!input.is_raw_mouse_capture_enabled());
    assert!(input.enable_raw_mouse_capture());
    assert!(!input.enable_raw_mouse_capture());

    let move_evt = SystemEvent::Mouse(MouseEvent {
      kind: MouseEventKind::Move,
      button: None,
      scroll: None,
      x: 10,
      y: 5,
    });
    let scroll_evt = SystemEvent::Mouse(MouseEvent {
      kind: MouseEventKind::Scroll,
      button: None,
      scroll: Some(ScrollDirection::Down),
      x: 20,
      y: 10,
    });
    input.system_sender.send(move_evt).unwrap();
    input.system_sender.send(scroll_evt).unwrap();

    let system_events = input.drain_system_events();
    assert_eq!(system_events, vec![move_evt, scroll_evt]);

    let raw_mouse = input.take_raw_mouse_events();
    assert_eq!(raw_mouse.len(), 2);
    assert!(matches!(raw_mouse[0].kind, MouseEventKind::Move));
    assert!(matches!(raw_mouse[1].kind, MouseEventKind::Scroll));

    assert!(input.take_raw_mouse_events().is_empty());
  }

  #[test]
  fn raw_mouse_capture_respects_focus() {
    let mut input = InputService::new();
    input.enable_raw_mouse_capture();

    input.focused = false;
    input
      .system_sender
      .send(SystemEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Move,
        button: None,
        scroll: None,
        x: 1,
        y: 1,
      }))
      .unwrap();
    input.drain_system_events();
    assert!(input.take_raw_mouse_events().is_empty());

    input.focused = true;
    input
      .system_sender
      .send(SystemEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Move,
        button: None,
        scroll: None,
        x: 2,
        y: 2,
      }))
      .unwrap();
    input.drain_system_events();
    assert_eq!(input.take_raw_mouse_events().len(), 1);
  }

  #[test]
  fn raw_mouse_disable_preserves_events_and_enable_starts_clean() {
    let mut input = InputService::new();
    input.enable_raw_mouse_capture();
    input
      .system_sender
      .send(SystemEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Move,
        button: None,
        scroll: None,
        x: 3,
        y: 3,
      }))
      .unwrap();
    input.drain_system_events();
    assert!(input.disable_raw_mouse_capture());
    assert!(!input.disable_raw_mouse_capture());
    assert_eq!(input.take_raw_mouse_events().len(), 1);

    input.enable_raw_mouse_capture();
    assert!(input.take_raw_mouse_events().is_empty());
  }
}
