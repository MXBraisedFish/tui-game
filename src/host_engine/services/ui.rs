use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

use super::TextInputEvent;
use super::hit_area::{HitAreaEvent, HitAreaId, HitAreaObjects};
use super::input::InputActionEvent;
use super::scroll_box::ScrollBoxObjects;
use super::slice::SliceObjects;
use super::surface::SurfaceId;
use super::text_input::TextInputObjects;

static NEXT_POOL_ID: AtomicU64 = AtomicU64::new(1);

/// UI 服务（无状态标记类型）
#[derive(Clone, Debug)]
pub struct UiService;

impl UiService {
  pub fn new() -> Self {
    Self
  }
}

/// UI 对象池，存储所有 UI 组件的共享状态
pub struct UiObjectPool {
  id: u64,
  render_order: u64,
  pub(crate) events: VecDeque<UiComponentEvent>,
  pub(crate) surfaces: Vec<SurfaceId>,
  pub(crate) hit_areas: HitAreaObjects,
  pub(crate) text_inputs: TextInputObjects,
  pub(crate) slices: SliceObjects,
  pub(crate) scroll_boxes: ScrollBoxObjects,
}

/// UI 事件（动作 / 点击区域 / 文本输入）
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
      surfaces: Vec::new(),
      hit_areas: HitAreaObjects::new(),
      text_inputs: TextInputObjects::new(),
      slices: SliceObjects::new(),
      scroll_boxes: ScrollBoxObjects::new(),
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

  pub(crate) fn surface_exists(&self, surface: SurfaceId) -> bool {
    match surface {
      SurfaceId::Slice(id) => self.slices.slices.contains_key(&id),
      SurfaceId::ScrollBox(id) => self.scroll_boxes.boxes.contains_key(&id),
    }
  }

  pub(crate) fn move_surface_to_edge(&mut self, surface: SurfaceId, back: bool) -> bool {
    let Some(index) = self.surfaces.iter().position(|current| *current == surface) else {
      return false;
    };
    self.surfaces.remove(index);
    if back {
      self.surfaces.insert(0, surface);
    } else {
      self.surfaces.push(surface);
    }
    true
  }

  pub(crate) fn move_surface_relative(
    &mut self,
    surface: SurfaceId,
    target: SurfaceId,
    above: bool,
  ) -> bool {
    if surface == target || !self.surface_exists(surface) || !self.surface_exists(target) {
      return false;
    }
    self.surfaces.retain(|current| *current != surface);
    let Some(target_index) = self.surfaces.iter().position(|current| *current == target) else {
      return false;
    };
    self
      .surfaces
      .insert(target_index + usize::from(above), surface);
    true
  }
}

/// UI 对象池持有者 trait
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
