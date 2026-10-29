//! UI and runtime object pools owned by exactly one Lua session.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::{Rc, Weak};

mod input;
mod link;
mod markdown;
pub(crate) use markdown::LuaMarkdownBinding;
mod scene;
mod scrollbox;
pub(crate) use input::LuaInputBinding;
pub(crate) use link::LuaLinkBinding;
pub(crate) use scrollbox::{LuaScrollboxBinding, SurfacePlacement};

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
  HitAreaId, HitAreaOptions, HitAreaService, RuntimeObjectPool, RuntimeObjectPoolOwner,
  SliceService, UiObjectPool, UiObjectPoolOwner,
};

/// Configuration retained by a session-owned script hitbox.
///
/// # Fields
///
/// * `width` - The configured width in terminal columns.
/// * `height` - The configured height in terminal rows.
/// * `options` - The hover and drag event switches.
/// * `placement` - The original origins retained by the latest drawing submission.
#[derive(Clone, Copy)]
pub(crate) struct LuaHitboxBinding {
  /// The configured width in terminal columns.
  pub width: u16,
  /// The configured height in terminal rows.
  pub height: u16,
  /// The hover and drag event switches.
  pub options: HitAreaOptions,
  /// The original coordinate origins retained from the last drawing submission.
  pub placement: Option<LuaHitboxPlacement>,
}

/// The original slice and hitbox origins used to report pointer coordinates after clipping.
#[derive(Clone, Copy)]
pub(crate) struct LuaHitboxPlacement {
  surface_origin: (i32, i32),
  position: (i32, i32),
}

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
  pub(crate) ui: UiObjectPool,
  runtime: RuntimeObjectPool,
  /// The timer parameters and callbacks retained by this session.
  pub(crate) timers: BTreeMap<ScheduledTimerId, LuaTimerBinding>,
  /// The hitbox configurations indexed by their pool-local native identity.
  pub(crate) hitboxes: BTreeMap<u64, LuaHitboxBinding>,
  /// The script input configurations and presentation options.
  pub(crate) inputs: BTreeMap<u64, LuaInputBinding>,
  /// The script links indexed by their pool-local native identities.
  pub(crate) links: BTreeMap<u64, LuaLinkBinding>,
  pub(crate) scrollboxes: BTreeMap<u64, LuaScrollboxBinding>,
  pub(crate) next_scrollbox_id: u64,
  pub(crate) markdowns: BTreeMap<u64, LuaMarkdownBinding>,
  pub(crate) next_markdown_id: u64,
  pub(crate) surface_placements: HashMap<tg_service_widget::SliceId, SurfacePlacement>,
  pub(crate) surface_sequence: u64,
  pub(crate) slice_scopes: HashMap<tg_service_widget::SliceId, tg_service_widget::PointerScopeId>,
  /// The editing focus and pointer capture owned by this session.
  pub(crate) text_input: tg_service_widget::TextInputService,
}

impl LuaObjectPool {
  /// Create a Lua object pool with its initial state.
  pub fn new() -> Self {
    Self {
      ui: UiObjectPool::new(),
      runtime: RuntimeObjectPool::new(),
      timers: BTreeMap::new(),
      hitboxes: BTreeMap::new(),
      inputs: BTreeMap::new(),
      links: BTreeMap::new(),
      scrollboxes: BTreeMap::new(),
      next_scrollbox_id: 0,
      markdowns: BTreeMap::new(),
      next_markdown_id: 0,
      surface_placements: HashMap::new(),
      surface_sequence: 0,
      slice_scopes: HashMap::new(),
      text_input: tg_service_widget::TextInputService::new(),
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

  /// Register an existing hitbox in its owning surface for the current frame.
  ///
  /// # Arguments
  ///
  /// * `id` - The pool-local hitbox identity.
  /// * `target` - The owning base or slice surface.
  /// * `position` - The signed column and row in the original target surface.
  /// * `canvas` - The canvas supplying viewport clipping.
  pub fn draw_hitbox(
    &mut self,
    id: u64,
    target: crate::LuaDrawTarget,
    position: (i32, i32),
    canvas: &tg_service_canvas::CanvasService,
  ) {
    if let Some(binding) = self.hitboxes.get_mut(&id) {
      let size = tg_service_layout::Size {
        width: binding.width,
        height: binding.height,
      };
      let origin = match target {
        crate::LuaDrawTarget::Base => {
          HitAreaService::new().render_at(&mut self.ui, HitAreaId(id), position, size, canvas);
          Some((
            i32::from(canvas.viewport().x),
            i32::from(canvas.viewport().y),
          ))
        }
        crate::LuaDrawTarget::Slice(slice) => {
          HitAreaService::new().render_at_on(
            &mut self.ui,
            HitAreaId(id),
            slice,
            position,
            size,
            canvas,
          );
          canvas.slice_origin(slice)
        }
      };
      binding.placement = origin.map(|surface_origin| LuaHitboxPlacement {
        surface_origin,
        position,
      });
    }
  }

  /// Route a pointer through session widgets while retaining the original hitbox entry point.
  ///
  /// # Arguments
  ///
  /// * `canvas` - The canvas used to register this session's hit regions.
  /// * `event` - The physical terminal pointer event.
  pub fn route_hitbox_mouse(
    &mut self,
    canvas: &tg_service_canvas::CanvasService,
    event: tg_core_input::MouseEvent,
  ) -> Vec<LuaEventData> {
    self.route_widget_mouse(canvas, event)
  }

  /// Route a physical pointer event through this session's submitted hitboxes and inputs.
  pub fn route_widget_mouse(
    &mut self,
    canvas: &tg_service_canvas::CanvasService,
    event: tg_core_input::MouseEvent,
  ) -> Vec<LuaEventData> {
    let mut boxes = self
      .scrollboxes
      .iter()
      .filter_map(|(id, binding)| binding.widget.frame_order().map(|order| (*id, order)))
      .collect::<Vec<_>>();
    boxes.sort_by_key(|(_, order)| std::cmp::Reverse(*order));
    for (id, _) in boxes {
      if self
        .scrollboxes
        .get_mut(&id)
        .unwrap()
        .widget
        .route_mouse(&self.ui, event)
      {
        break;
      }
    }
    let event_point = (event.x, event.y);
    tg_service_widget::MarkdownService::new().route_mouse_event(
      &mut self.ui,
      &self.text_input,
      event,
    );
    tg_service_widget::HyperlinkService::new().route_mouse_event(
      &mut self.ui,
      &self.text_input,
      event,
    );
    HitAreaService::new().route_mouse_event(&mut self.ui, &mut self.text_input, canvas, event);
    let mut events = Vec::new();
    while let Some(event) = self.ui.pop_event() {
      if let tg_service_widget::UiEvent::Hyperlink(event) = &event
        && let Some(translated) = self.translate_link_event(event.clone())
      {
        events.push(translated);
      }
      if let tg_service_widget::UiEvent::Markdown(event) = &event
        && let Some(event) = self.translate_markdown_event(event.clone())
      {
        events.push(event);
      }
      if let tg_service_widget::UiEvent::TextInput(event) = &event
        && let Some(translated) = self.translate_input_event(event.clone())
      {
        events.push(translated);
      }
      if let tg_service_widget::UiEvent::HitArea(event) = event {
        let id = match &event {
          tg_service_widget::HitAreaEvent::HoverEnter { id, .. }
          | tg_service_widget::HitAreaEvent::HoverMove { id, .. }
          | tg_service_widget::HitAreaEvent::HoverLeave { id, .. }
          | tg_service_widget::HitAreaEvent::Press { id, .. }
          | tg_service_widget::HitAreaEvent::Release { id, .. }
          | tg_service_widget::HitAreaEvent::Click { id, .. }
          | tg_service_widget::HitAreaEvent::Drag { id, .. } => id.0,
        };
        if let Some(placement) = self.hitboxes.get(&id).and_then(|binding| binding.placement) {
          let mut translated = crate::events::translate::translate_hit_area_event(id, &event);
          if let LuaEventData::HitArea(data) = &mut translated {
            data.x = i32::from(event_point.0) - placement.surface_origin.0;
            data.y = i32::from(event_point.1) - placement.surface_origin.1;
            data.relative_x = i64::from(data.x) - i64::from(placement.position.0);
            data.relative_y = i64::from(data.y) - i64::from(placement.position.1);
          }
          events.push(translated);
        }
      }
    }
    events.extend(self.take_scrollbox_events());
    events
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
