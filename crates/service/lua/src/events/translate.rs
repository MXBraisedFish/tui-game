//! Translation of service events into the Lua session event model.

use tg_service_animation::{AnimationEvent, AnimationEventKind};
use tg_service_time::RepeatTimerEvent;
use tg_service_widget::{HitAreaEvent, ScrollBoxEvent};

use super::{
  LuaAnimationEvent, LuaAnimationEventKind, LuaEventData, LuaHitAreaEvent, LuaScrollBoxEvent,
  LuaTimerEvent, LuaTimerEventKind, LuaTimerKind,
};

/// Convert a repeating-timer outcome into its session-local script event.
pub fn translate_repeat_timer_event(lua_id: u64, event: RepeatTimerEvent) -> LuaEventData {
  match event {
    RepeatTimerEvent::Tick { executed_count, .. } => LuaEventData::Timer(LuaTimerEvent {
      id: lua_id,
      timer_kind: LuaTimerKind::Repeat,
      kind: LuaTimerEventKind::Tick,
      executed_count: Some(executed_count),
      object_id: None,
      tip: None,
      revision: None,
    }),
    RepeatTimerEvent::Finished { executed_count, .. } => LuaEventData::Timer(LuaTimerEvent {
      id: lua_id,
      timer_kind: LuaTimerKind::Repeat,
      kind: LuaTimerEventKind::Finished,
      executed_count: Some(executed_count),
      object_id: None,
      tip: None,
      revision: None,
    }),
  }
}

/// Convert an animation outcome into its session-local script event.
pub fn translate_animation_event(lua_id: u64, event: &AnimationEvent) -> LuaEventData {
  let kind = match &event.kind {
    AnimationEventKind::Started => LuaAnimationEventKind::Started,
    AnimationEventKind::Marker { name } => LuaAnimationEventKind::Marker { name: name.clone() },
    AnimationEventKind::Loop { completed } => LuaAnimationEventKind::Loop {
      completed: *completed,
    },
    AnimationEventKind::Finished => LuaAnimationEventKind::Finished,
    AnimationEventKind::Cancelled => LuaAnimationEventKind::Cancelled,
  };
  LuaEventData::Animation(LuaAnimationEvent { id: lua_id, kind })
}

/// Convert a pointer-region outcome into its script-visible hit-area event.
pub fn translate_hit_area_event(lua_id: u64, event: &HitAreaEvent) -> LuaEventData {
  let (kind, x, y, button, dx, dy) = match event {
    HitAreaEvent::HoverEnter { x, y, .. } => ("hover_enter", *x, *y, None, None, None),
    HitAreaEvent::HoverMove { x, y, .. } => ("hover_move", *x, *y, None, None, None),
    HitAreaEvent::HoverLeave { x, y, .. } => ("hover_leave", *x, *y, None, None, None),
    HitAreaEvent::Press { button, x, y, .. } => {
      ("press", *x, *y, Some(mouse_button(*button)), None, None)
    }
    HitAreaEvent::Release { button, x, y, .. } => {
      ("release", *x, *y, Some(mouse_button(*button)), None, None)
    }
    HitAreaEvent::Click { button, x, y, .. } => {
      ("click", *x, *y, Some(mouse_button(*button)), None, None)
    }
    HitAreaEvent::Drag {
      button,
      x,
      y,
      dx,
      dy,
      ..
    } => (
      "drag",
      *x,
      *y,
      Some(mouse_button(*button)),
      Some(*dx),
      Some(*dy),
    ),
  };
  LuaEventData::HitArea(LuaHitAreaEvent {
    id: lua_id,
    kind,
    x,
    y,
    button,
    dx,
    dy,
  })
}

/// Convert a scroll-position change into its script-visible scroll-box event.
pub fn translate_scroll_box_event(lua_id: u64, event: ScrollBoxEvent) -> LuaEventData {
  match event {
    ScrollBoxEvent::Scrolled { x, y, .. } => {
      LuaEventData::ScrollBox(LuaScrollBoxEvent { id: lua_id, x, y })
    }
  }
}

fn mouse_button(button: tg_core_input::MouseButton) -> &'static str {
  match button {
    tg_core_input::MouseButton::Left => "left",
    tg_core_input::MouseButton::Middle => "middle",
    tg_core_input::MouseButton::Right => "right",
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tg_core_input::MouseButton;
  use tg_service_animation::AnimationId;
  use tg_service_time::RepeatTimerId;
  use tg_service_widget::{HitAreaId, ScrollBoxId};

  #[test]
  fn translators_use_lua_local_ids_instead_of_host_ids() {
    let repeat = translate_repeat_timer_event(
      91,
      RepeatTimerEvent::Tick {
        id: RepeatTimerId(500),
        executed_count: 3,
      },
    );
    assert!(matches!(
      repeat,
      LuaEventData::Timer(LuaTimerEvent {
        id: 91,
        executed_count: Some(3),
        object_id: None,
        tip: None,
        revision: None,
        ..
      })
    ));

    let hit = translate_hit_area_event(
      7,
      &HitAreaEvent::Click {
        id: HitAreaId(800),
        button: MouseButton::Left,
        x: 4,
        y: 5,
      },
    );
    assert!(matches!(
      hit,
      LuaEventData::HitArea(LuaHitAreaEvent { id: 7, .. })
    ));

    let animation = AnimationEvent {
      id: AnimationId::new(12, 34),
      kind: AnimationEventKind::Finished,
    };
    assert!(matches!(
      translate_animation_event(3, &animation),
      LuaEventData::Animation(LuaAnimationEvent { id: 3, .. })
    ));

    let scroll = translate_scroll_box_event(
      2,
      ScrollBoxEvent::Scrolled {
        id: ScrollBoxId(99),
        x: 8,
        y: 9,
      },
    );
    assert!(matches!(
      scroll,
      LuaEventData::ScrollBox(LuaScrollBoxEvent { id: 2, x: 8, y: 9 })
    ));
  }
}
