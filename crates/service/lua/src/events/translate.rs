use tg_service_animation::{AnimationEvent, AnimationEventKind};
use tg_service_time::{DelayTimerEvent, RepeatTimerEvent, TimerEvent};
use tg_service_widget::{HitAreaEvent, HyperlinkEvent, MarkdownEvent, ScrollBoxEvent, TextInputEvent};

use super::{
  LuaAnimationEvent, LuaAnimationEventKind, LuaEventData, LuaHitAreaEvent, LuaHyperlinkEvent,
  LuaMarkdownEvent, LuaScrollBoxEvent, LuaTextInputEvent, LuaTimerEvent, LuaTimerEventKind,
  LuaTimerKind,
};

pub fn translate_timer_event(lua_id: u64, event: TimerEvent) -> LuaEventData {
  match event {
    TimerEvent::Finished { .. } => LuaEventData::Timer(LuaTimerEvent {
      id: lua_id,
      timer_kind: LuaTimerKind::Timer,
      kind: LuaTimerEventKind::Finished,
      executed_count: None,
    }),
  }
}

pub fn translate_delay_timer_event(lua_id: u64, event: DelayTimerEvent) -> LuaEventData {
  match event {
    DelayTimerEvent::Finished { .. } => LuaEventData::Timer(LuaTimerEvent {
      id: lua_id,
      timer_kind: LuaTimerKind::Delay,
      kind: LuaTimerEventKind::Finished,
      executed_count: None,
    }),
  }
}

pub fn translate_repeat_timer_event(lua_id: u64, event: RepeatTimerEvent) -> LuaEventData {
  match event {
    RepeatTimerEvent::Tick { executed_count, .. } => LuaEventData::Timer(LuaTimerEvent {
      id: lua_id,
      timer_kind: LuaTimerKind::Repeat,
      kind: LuaTimerEventKind::Tick,
      executed_count: Some(executed_count),
    }),
    RepeatTimerEvent::Finished { executed_count, .. } => LuaEventData::Timer(LuaTimerEvent {
      id: lua_id,
      timer_kind: LuaTimerKind::Repeat,
      kind: LuaTimerEventKind::Finished,
      executed_count: Some(executed_count),
    }),
  }
}

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

pub fn translate_hyperlink_event(lua_id: u64, event: &HyperlinkEvent) -> LuaEventData {
  match event {
    HyperlinkEvent::Clicked { link, .. } => LuaEventData::Hyperlink(LuaHyperlinkEvent {
      id: lua_id,
      link: link.clone(),
    }),
  }
}

pub fn translate_markdown_event(lua_id: u64, event: &MarkdownEvent) -> LuaEventData {
  match event {
    MarkdownEvent::LinkClicked { href, text, .. } => LuaEventData::Markdown(LuaMarkdownEvent {
      id: lua_id,
      href: href.clone(),
      text: text.clone(),
    }),
  }
}

pub fn translate_text_input_event(lua_id: u64, event: &TextInputEvent) -> LuaEventData {
  let (kind, value) = match event {
    TextInputEvent::Focused { .. } => ("focused", None),
    TextInputEvent::Blurred { .. } => ("blurred", None),
    TextInputEvent::Changed { value, .. } => ("changed", Some(value.clone())),
    TextInputEvent::Submit { value, .. } => ("submit", Some(value.clone())),
    TextInputEvent::Cancel { value, .. } => ("cancel", Some(value.clone())),
    TextInputEvent::Pressed { .. } => ("pressed", None),
    TextInputEvent::PressedOutside { .. } => ("pressed_outside", None),
  };
  LuaEventData::TextInput(LuaTextInputEvent {
    id: lua_id,
    kind,
    value,
  })
}

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
