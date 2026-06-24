use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

use super::TextInputEvent;
use super::hit_area::{HitAreaEvent, HitAreaId, HitAreaObjects};
use super::input::InputActionEvent;
use super::slice::SliceObjects;
use super::text_input::TextInputObjects;

static NEXT_POOL_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub struct UiService;

impl UiService {
  pub fn new() -> Self {
    Self
  }
}

/// 页面级 UI 对象池。每个页面持有一个独立实例。
pub struct UiObjectPool {
  id: u64,
  render_order: u64,
  pub(crate) events: VecDeque<UiComponentEvent>,
  pub(crate) hit_areas: HitAreaObjects,
  pub(crate) text_inputs: TextInputObjects,
  pub(crate) slices: SliceObjects,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiEvent {
  Action(InputActionEvent),
  HitArea(HitAreaEvent),
  TextInput(TextInputEvent),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UiComponentEvent {
  HitArea(HitAreaEvent),
  TextInput(TextInputEvent),
}

impl UiComponentEvent {
  pub(crate) fn hit_area_id(&self) -> Option<HitAreaId> {
    match self {
      Self::HitArea(event) => Some(match event {
        HitAreaEvent::HoverEnter { id, .. }
        | HitAreaEvent::HoverMove { id, .. }
        | HitAreaEvent::HoverLeave { id, .. }
        | HitAreaEvent::Press { id, .. }
        | HitAreaEvent::Release { id, .. }
        | HitAreaEvent::Click { id, .. }
        | HitAreaEvent::Drag { id, .. } => *id,
      }),
      Self::TextInput(_) => None,
    }
  }

  pub(crate) fn text_input_id(&self) -> Option<super::TextInputId> {
    match self {
      Self::TextInput(event) => Some(match event {
        TextInputEvent::Focused { id }
        | TextInputEvent::Blurred { id }
        | TextInputEvent::Changed { id, .. }
        | TextInputEvent::Submit { id, .. }
        | TextInputEvent::Cancel { id, .. }
        | TextInputEvent::Pressed { id }
        | TextInputEvent::PressedOutside { id } => *id,
      }),
      Self::HitArea(_) => None,
    }
  }
}

impl UiObjectPool {
  pub fn new() -> Self {
    Self {
      id: NEXT_POOL_ID.fetch_add(1, Ordering::Relaxed),
      render_order: 0,
      events: VecDeque::new(),
      hit_areas: HitAreaObjects::new(),
      text_inputs: TextInputObjects::new(),
      slices: SliceObjects::new(),
    }
  }

  pub(crate) fn id(&self) -> u64 {
    self.id
  }

  pub(crate) fn next_render_order(&mut self) -> u64 {
    self.render_order += 1;
    self.render_order
  }

  pub(crate) fn begin_render(&mut self) {
    self.render_order = 0;
    self.hit_areas.clear_hits();
    self.text_inputs.clear_hits();
  }

  pub(crate) fn push_hit_event(&mut self, event: HitAreaEvent) {
    self.events.push_back(UiComponentEvent::HitArea(event));
  }

  pub(crate) fn push_text_event(&mut self, event: TextInputEvent) {
    self.events.push_back(UiComponentEvent::TextInput(event));
  }

  pub(crate) fn pop_event(&mut self) -> Option<UiEvent> {
    self.events.pop_front().map(|event| match event {
      UiComponentEvent::HitArea(event) => UiEvent::HitArea(event),
      UiComponentEvent::TextInput(event) => UiEvent::TextInput(event),
    })
  }
}

/// 所有有状态 Host UI 页面的对象池访问规范。
pub trait UiObjectPoolOwner {
  fn objects(&self) -> &UiObjectPool;
  fn objects_mut(&mut self) -> &mut UiObjectPool;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn pools_have_distinct_internal_ids() {
    assert_ne!(UiObjectPool::new().id(), UiObjectPool::new().id());
  }
}
