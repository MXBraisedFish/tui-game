//! UI and runtime object pools owned by exactly one Lua session.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

use crate::{LuaEventData, LuaTimerEvent, LuaTimerEventKind, LuaTimerKind};
use mlua::Function;
use tg_service_time::{ScheduledTimerId, TimeService};

/// Script-facing configuration and optional event callback owned by one timer.
///
/// # Fields
///
/// * `duration` - The active duration in seconds.
/// * `delay` - The initial and recurring wait in seconds.
/// * `interval` - The additional wait between executions.
/// * `looping` - Whether repeated executions are enabled.
/// * `repetitions` - The optional total execution limit.
/// * `callback` - The function receiving timer events.
/// * `tip` - The custom event text.
#[derive(Clone)]
pub(crate) struct LuaTimerBinding {
  /// The active duration in seconds.
  pub duration: f64,
  /// The initial and recurring delay in seconds.
  pub delay: f64,
  /// The additional recurring gap in seconds.
  pub interval: f64,
  /// Whether more than one execution is enabled.
  pub looping: bool,
  /// The optional total number of executions when looping.
  pub repetitions: Option<u32>,
  /// The callback receiving timer events instead of HandleEvent.
  pub callback: Option<Function>,
  /// The custom text attached to timer events.
  pub tip: Option<String>,
}

use tg_service_widget::{
  RuntimeObjectPool, RuntimeObjectPoolOwner, SliceService, UiObjectPool, UiObjectPoolOwner,
};

/// The shared type used for shared Lua object pool.
pub(crate) type SharedLuaObjectPool = Rc<RefCell<Option<LuaObjectPool>>>;
/// The shared type used for weak Lua object pool.
pub(crate) type WeakLuaObjectPool = Weak<RefCell<Option<LuaObjectPool>>>;

/// Create shared access to a new session-owned UI and runtime object pool.
pub(crate) fn shared_lua_object_pool() -> SharedLuaObjectPool {
  Rc::new(RefCell::new(Some(LuaObjectPool::new())))
}

/// UI and runtime objects owned by exactly one Lua session.
pub struct LuaObjectPool {
  ui: UiObjectPool,
  runtime: RuntimeObjectPool,
  /// The timer parameters and callbacks retained by this session.
  pub(crate) timers: BTreeMap<ScheduledTimerId, LuaTimerBinding>,
}

impl LuaObjectPool {
  /// Create a Lua object pool with its initial state.
  pub fn new() -> Self {
    Self {
      ui: UiObjectPool::new(),
      runtime: RuntimeObjectPool::new(),
      timers: BTreeMap::new(),
    }
  }

  /// Return the current ui.
  pub fn ui(&self) -> &UiObjectPool {
    &self.ui
  }

  /// Return mutable access to the owned ui.
  pub fn ui_mut(&mut self) -> &mut UiObjectPool {
    &mut self.ui
  }

  /// Return the current runtime.
  pub fn runtime(&self) -> &RuntimeObjectPool {
    &self.runtime
  }

  /// Return mutable access to the owned runtime.
  pub fn runtime_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime
  }

  /// Drain timer completions for submission to the owning session's event broker.
  ///
  /// Advance the runtime time pool with [`TimeService::update`] before calling this method.
  /// Deliver returned events through [`crate::LuaSession::dispatch_event`] so callbacks use
  /// execution budgets and invalidated timer events are discarded.
  pub fn take_timer_events(&mut self) -> Vec<LuaEventData> {
    TimeService::new()
      .take_scheduled_timer_events(&mut self.runtime.time)
      .into_iter()
      .filter_map(|event| {
        let binding = self.timers.get(&event.id)?;
        Some(LuaEventData::Timer(LuaTimerEvent {
          id: event.id.0,
          timer_kind: LuaTimerKind::Timer,
          kind: if event.finished {
            LuaTimerEventKind::Finished
          } else {
            LuaTimerEventKind::Tick
          },
          executed_count: Some(event.executed_count),
          object_id: Some(format!("timer_{:03}", event.id.0)),
          tip: binding.tip.clone(),
          revision: Some(event.revision),
        }))
      })
      .collect()
  }

  /// Prepare per-frame state and discard submissions or observations belonging to the previous
  /// frame.
  pub fn begin_frame(&mut self) {
    SliceService::new().begin_frame(&mut self.ui);
  }
}

impl Default for LuaObjectPool {
  fn default() -> Self {
    Self::new()
  }
}

impl UiObjectPoolOwner for LuaObjectPool {
  fn objects(&self) -> &UiObjectPool {
    self.ui()
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    self.ui_mut()
  }
}

impl RuntimeObjectPoolOwner for LuaObjectPool {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    self.runtime()
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    self.runtime_mut()
  }
}
