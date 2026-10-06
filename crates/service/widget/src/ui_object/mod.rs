//! Shared UI object ownership, surface ordering, and queued interaction events.

pub(crate) mod interactives;
pub(crate) mod surfaces;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

use interactives::hit_area::{HitAreaEvent, HitAreaId, HitAreaObjects};
use interactives::hyperlink::{HyperlinkEvent, HyperlinkId, HyperlinkObjects};
use interactives::text_input::{TextInputEvent, TextInputObjects};
use surfaces::markdown_view::{MarkdownEvent, MarkdownViewId, MarkdownViewObjects};
use surfaces::progress_bar::ProgressBarObjects;
use surfaces::scroll_box::{ScrollBoxEvent, ScrollBoxObjects};
use surfaces::slice::SliceObjects;
use surfaces::surface::SurfaceId;
use surfaces::table::TableObjects;
use tg_core_audio::AudioPoolId;
use tg_core_input::InputActionEvent;
use tg_service_audio::AudioObjectPool;

static NEXT_POOL_ID: AtomicU64 = AtomicU64::new(1);

/// Shared UI object state, surface ordering, and interaction events owned by one view or session.
///
/// # Fields
///
/// * `events` - Pending events retained in delivery order.
/// * `surfaces` - The ordered surfaces retained by this owner.
/// * `hit_areas` - The hit areas.
/// * `hyperlinks` - The hyperlinks.
/// * `markdown_views` - The markdown views.
/// * `text_inputs` - The text inputs.
/// * `slices` - The slices.
/// * `scroll_boxes` - The scroll boxes.
/// * `progress_bars` - The progress bars.
/// * `tables` - The tables.
pub struct UiObjectPool {
  id: u64,
  audio: AudioObjectPool,
  render_order: u64,
  /// Pending events retained in delivery order.
  pub(crate) events: VecDeque<UiComponentEvent>,
  /// The ordered surfaces retained by this owner.
  pub(crate) surfaces: Vec<SurfaceId>,
  /// The hit areas.
  pub(crate) hit_areas: HitAreaObjects,
  /// The hyperlinks.
  pub(crate) hyperlinks: HyperlinkObjects,
  /// The markdown views.
  pub(crate) markdown_views: MarkdownViewObjects,
  /// The text inputs.
  pub(crate) text_inputs: TextInputObjects,
  /// The slices.
  pub(crate) slices: SliceObjects,
  /// The scroll boxes.
  pub scroll_boxes: ScrollBoxObjects,
  /// The progress bars.
  pub(crate) progress_bars: ProgressBarObjects,
  /// The tables.
  pub(crate) tables: TableObjects,
}

/// A UI event payload queued for its owning consumer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiEvent {
  /// A action notification delivered to the owning consumer.
  Action(InputActionEvent),
  /// A hit area notification delivered to the owning consumer.
  HitArea(HitAreaEvent),
  /// A hyperlink notification delivered to the owning consumer.
  Hyperlink(HyperlinkEvent),
  /// A markdown notification delivered to the owning consumer.
  Markdown(MarkdownEvent),
  /// A text input notification delivered to the owning consumer.
  TextInput(TextInputEvent),
  /// A scroll box notification delivered to the owning consumer.
  ScrollBox(ScrollBoxEvent),
}

/// A UI component event payload queued for its owning consumer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UiComponentEvent {
  /// A hit area notification delivered to the owning consumer.
  HitArea(HitAreaEvent),
  /// A hyperlink notification delivered to the owning consumer.
  Hyperlink(HyperlinkEvent),
  /// A markdown notification delivered to the owning consumer.
  Markdown(MarkdownEvent),
  /// A text input notification delivered to the owning consumer.
  TextInput(TextInputEvent),
}

impl UiComponentEvent {
  /// Return the current hit area id.
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
      Self::Hyperlink(_) | Self::Markdown(_) | Self::TextInput(_) => None,
    }
  }

  /// Return the current hyperlink id.
  pub(crate) fn hyperlink_id(&self) -> Option<HyperlinkId> {
    match self {
      Self::Hyperlink(HyperlinkEvent::Clicked { id, .. }) => Some(*id),
      Self::HitArea(_) | Self::Markdown(_) | Self::TextInput(_) => None,
    }
  }

  /// Return the current markdown id.
  pub(crate) fn markdown_id(&self) -> Option<MarkdownViewId> {
    match self {
      Self::Markdown(MarkdownEvent::LinkClicked { id, .. }) => Some(*id),
      Self::HitArea(_) | Self::Hyperlink(_) | Self::TextInput(_) => None,
    }
  }

  /// Return the current text input id.
  pub(crate) fn text_input_id(&self) -> Option<interactives::text_input::TextInputId> {
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
      Self::HitArea(_) | Self::Hyperlink(_) | Self::Markdown(_) => None,
    }
  }
}

impl UiObjectPool {
  /// Create a UI object pool with its initial state.
  pub fn new() -> Self {
    let id = NEXT_POOL_ID.fetch_add(1, Ordering::Relaxed);
    Self {
      id,
      audio: AudioObjectPool::new(AudioPoolId(id)),
      render_order: 0,
      events: VecDeque::new(),
      surfaces: Vec::new(),
      hit_areas: HitAreaObjects::new(),
      hyperlinks: HyperlinkObjects::new(),
      markdown_views: MarkdownViewObjects::new(),
      text_inputs: TextInputObjects::new(),
      slices: SliceObjects::new(),
      scroll_boxes: ScrollBoxObjects::new(),
      progress_bars: ProgressBarObjects::new(),
      tables: TableObjects::new(),
    }
  }

  /// Return the current id.
  pub fn id(&self) -> u64 {
    self.id
  }

  /// Return the current audio.
  pub fn audio(&self) -> &AudioObjectPool {
    &self.audio
  }

  /// Return mutable access to the owned audio.
  pub fn audio_mut(&mut self) -> &mut AudioObjectPool {
    &mut self.audio
  }

  /// Allocate the next drawing-order value for hit-test precedence.
  pub(crate) fn next_render_order(&mut self) -> u64 {
    self.render_order += 1;
    self.render_order
  }

  /// Reset drawing order and per-frame interaction geometry before rendering the pool.
  pub fn begin_render(&mut self) {
    self.render_order = 0;
    self.hit_areas.clear_hits();
    self.hyperlinks.clear_hits();
    self.markdown_views.clear_hits();
    self.text_inputs.clear_hits();
  }

  /// Queue the hit event in delivery order.
  pub(crate) fn push_hit_event(&mut self, event: HitAreaEvent) {
    self.events.push_back(UiComponentEvent::HitArea(event));
  }

  /// Queue the hyperlink event in delivery order.
  pub(crate) fn push_hyperlink_event(&mut self, event: HyperlinkEvent) {
    self.events.push_back(UiComponentEvent::Hyperlink(event));
  }

  /// Queue the markdown event in delivery order.
  pub(crate) fn push_markdown_event(&mut self, event: MarkdownEvent) {
    self.events.push_back(UiComponentEvent::Markdown(event));
  }

  /// Queue the text event in delivery order.
  pub(crate) fn push_text_event(&mut self, event: TextInputEvent) {
    self.events.push_back(UiComponentEvent::TextInput(event));
  }

  /// Pop the next queued UI interaction event.
  pub fn pop_event(&mut self) -> Option<UiEvent> {
    self.events.pop_front().map(|event| match event {
      UiComponentEvent::HitArea(event) => UiEvent::HitArea(event),
      UiComponentEvent::Hyperlink(event) => UiEvent::Hyperlink(event),
      UiComponentEvent::Markdown(event) => UiEvent::Markdown(event),
      UiComponentEvent::TextInput(event) => UiEvent::TextInput(event),
    })
  }

  /// Report whether the pool contains the specified drawing surface.
  pub(crate) fn surface_exists(&self, surface: SurfaceId) -> bool {
    match surface {
      SurfaceId::Slice(id) => self.slices.slices.contains_key(&id),
      SurfaceId::ScrollBox(id) => self.scroll_boxes.boxes.contains_key(&id),
    }
  }

  /// Move a surface to the front or back of the pool's ordered surface list.
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

  /// Move a surface relative to a peer while retaining valid pool ordering.
  ///
  /// # Arguments
  ///
  /// * `surface` - The destination drawing surface and its clipping bounds.
  /// * `target` - The object or resource affected by the operation.
  /// * `above` - The above.
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

impl Default for UiObjectPool {
  fn default() -> Self {
    Self::new()
  }
}

/// The contract for accessing or implementing UI object pool owner.
pub trait UiObjectPoolOwner {
  /// Return the current objects.
  fn objects(&self) -> &UiObjectPool;
  /// Return mutable access to the owned objects.
  fn objects_mut(&mut self) -> &mut UiObjectPool;
}
