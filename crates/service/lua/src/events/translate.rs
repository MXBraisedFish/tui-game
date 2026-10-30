//! Translation of service events into the Lua session event model.

#[cfg(test)]
use tg_service_animation::{AnimationEvent, AnimationEventKind};
#[cfg(test)]
use tg_service_time::RepeatTimerEvent;
use tg_service_widget::HitAreaEvent;
#[cfg(test)]
use tg_service_widget::ScrollBoxEvent;

#[cfg(test)]
use super::{
  LuaAnimationEvent, LuaAnimationEventKind, LuaScrollBoxEvent, LuaTimerEvent, LuaTimerEventKind,
  LuaTimerKind,
};
use super::{LuaEventData, LuaHitAreaEvent};

/// Convert a repeating-timer outcome into its session-local script event.
#[cfg(test)]
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
#[cfg(test)]
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
    HitAreaEvent::HoverEnter { x, y, .. } => ("hover_entered", *x, *y, None, None, None),
    HitAreaEvent::HoverMove { x, y, .. } => ("hover_moved", *x, *y, None, None, None),
    HitAreaEvent::HoverLeave { x, y, .. } => ("hover_left", *x, *y, None, None, None),
    HitAreaEvent::Press { button, x, y, .. } => {
      ("pressed", *x, *y, Some(mouse_button(*button)), None, None)
    }
    HitAreaEvent::Release { button, x, y, .. } => {
      ("released", *x, *y, Some(mouse_button(*button)), None, None)
    }
    HitAreaEvent::Click { button, x, y, .. } => {
      ("clicked", *x, *y, Some(mouse_button(*button)), None, None)
    }
    HitAreaEvent::Drag {
      button,
      x,
      y,
      dx,
      dy,
      ..
    } => (
      "dragged",
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
    x: i32::from(x),
    y: i32::from(y),
    relative_x: i64::from(x),
    relative_y: i64::from(y),
    button,
    dx,
    dy,
  })
}

/// Convert a scroll-position change into its script-visible scroll-box event.
#[cfg(test)]
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
  fn hitbox_notifications_use_past_tense_for_every_transition() {
    let id = HitAreaId(1);
    let button = MouseButton::Left;
    for (event, expected) in [
      (HitAreaEvent::HoverEnter { id, x: 2, y: 3 }, "hover_entered"),
      (HitAreaEvent::HoverMove { id, x: 2, y: 3 }, "hover_moved"),
      (HitAreaEvent::HoverLeave { id, x: 2, y: 3 }, "hover_left"),
      (
        HitAreaEvent::Press {
          id,
          button,
          x: 2,
          y: 3,
        },
        "pressed",
      ),
      (
        HitAreaEvent::Release {
          id,
          button,
          x: 2,
          y: 3,
        },
        "released",
      ),
      (
        HitAreaEvent::Click {
          id,
          button,
          x: 2,
          y: 3,
        },
        "clicked",
      ),
      (
        HitAreaEvent::Drag {
          id,
          button,
          x: 2,
          y: 3,
          dx: 1,
          dy: -1,
        },
        "dragged",
      ),
    ] {
      let LuaEventData::HitArea(data) = translate_hit_area_event(1, &event) else {
        panic!("hitbox event")
      };
      assert_eq!(data.kind, expected);
    }
  }

  #[test]
  fn renamed_hover_motion_coalesces_without_merging_button_events() {
    let hover = translate_hit_area_event(
      1,
      &HitAreaEvent::HoverMove {
        id: HitAreaId(1),
        x: 2,
        y: 3,
      },
    );
    let newer_hover = translate_hit_area_event(
      1,
      &HitAreaEvent::HoverMove {
        id: HitAreaId(1),
        x: 3,
        y: 4,
      },
    );
    assert!(hover.is_coalescible_with(&newer_hover));
    let other_hover = translate_hit_area_event(
      2,
      &HitAreaEvent::HoverMove {
        id: HitAreaId(2),
        x: 3,
        y: 4,
      },
    );
    assert!(!hover.is_coalescible_with(&other_hover));
    for event in [
      HitAreaEvent::Press {
        id: HitAreaId(1),
        button: MouseButton::Left,
        x: 2,
        y: 3,
      },
      HitAreaEvent::Drag {
        id: HitAreaId(1),
        button: MouseButton::Left,
        x: 2,
        y: 3,
        dx: 1,
        dy: 0,
      },
    ] {
      let event = translate_hit_area_event(1, &event);
      assert!(!event.is_coalescible_with(&event));
    }
  }

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
