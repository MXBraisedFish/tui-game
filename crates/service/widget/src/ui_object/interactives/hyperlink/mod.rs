//! Link text, clickable geometry, and pointer event routing.

mod state;
mod target;
mod types;
pub use target::{is_valid_link_target, open_link_target, resolve_link_target};

pub(crate) use self::state::HyperlinkObjects;
use self::state::{HyperlinkHit, HyperlinkState};
pub use self::types::{HyperlinkEvent, HyperlinkId, HyperlinkOptions};
use crate::UiObjectPool;
use crate::{SliceId, TextInputService};
use tg_core_input::{MouseButton, MouseEvent, MouseEventKind};
use tg_core_style::TextStyle;
use tg_service_canvas::CanvasService;
use tg_service_layout::Rect;
use tg_service_text_layout::{self as text_layout, DrawTextParams, TextWrapMode};

/// The public entry point for hyperlink operations.
#[derive(Default)]
pub struct HyperlinkService;

impl HyperlinkService {
  /// Create a hyperlink service with its initial state.
  pub fn new() -> Self {
    Self
  }

  /// Create an owned hyperlink object and return its identity.
  pub fn create(&self, pool: &mut UiObjectPool, options: HyperlinkOptions) -> Option<HyperlinkId> {
    validate_options(&options).then(|| {
      let id = HyperlinkId(pool.hyperlinks.next_id);
      pool.hyperlinks.next_id += 1;
      pool
        .hyperlinks
        .links
        .insert(id, HyperlinkState { options, hit: None });
      id
    })
  }

  /// Remove the identified widget object and release its owned state.
  pub fn remove(&self, pool: &mut UiObjectPool, id: HyperlinkId) -> bool {
    if pool.hyperlinks.links.remove(&id).is_none() {
      return false;
    }
    if pool.hyperlinks.hovered == Some(id) {
      pool.hyperlinks.hovered = None;
    }
    if pool.hyperlinks.pressed == Some(id) {
      pool.hyperlinks.pressed = None;
    }
    pool.events.retain(|event| event.hyperlink_id() != Some(id));
    true
  }

  /// Report whether the identified widget object is still present.
  pub fn exists(&self, pool: &UiObjectPool, id: HyperlinkId) -> bool {
    pool.hyperlinks.links.contains_key(&id)
  }

  /// Return the stored link destination for the identified hyperlink.
  pub fn link<'a>(&self, pool: &'a UiObjectPool, id: HyperlinkId) -> Option<&'a str> {
    Some(&pool.hyperlinks.links.get(&id)?.options.link)
  }

  /// Return the text for the addressed object when it is available.
  pub fn text<'a>(&self, pool: &'a UiObjectPool, id: HyperlinkId) -> Option<&'a str> {
    Some(&pool.hyperlinks.links.get(&id)?.options.text)
  }

  /// Update the link used by this hyperlink service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `link` - The link.
  pub fn set_link(&self, pool: &mut UiObjectPool, id: HyperlinkId, link: String) -> bool {
    if link.is_empty() {
      return false;
    }
    let Some(state) = pool.hyperlinks.links.get_mut(&id) else {
      return false;
    };
    state.options.link = link;
    true
  }

  /// Update the text used by this hyperlink service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `text` - The text to process or display.
  pub fn set_text(&self, pool: &mut UiObjectPool, id: HyperlinkId, text: String) -> bool {
    if text.is_empty() {
      return false;
    }
    let Some(state) = pool.hyperlinks.links.get_mut(&id) else {
      return false;
    };
    state.options.text = text;
    true
  }

  /// Render the hyperlink service into its requested terminal-cell surface.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `x` - The horizontal coordinate in terminal cells.
  /// * `y` - The vertical coordinate in terminal cells.
  /// * `canvas` - The clipped canvas used for drawing.
  pub fn render(
    &self,
    pool: &mut UiObjectPool,
    id: HyperlinkId,
    x: u16,
    y: u16,
    canvas: &mut CanvasService,
  ) -> bool {
    self.render_at(pool, id, None, (i32::from(x), i32::from(y)), canvas)
  }

  /// Render a link at unsigned coordinates in the specified slice.
  ///
  /// # Arguments
  ///
  /// * `pool` - The owning pool.
  /// * `id` - The link identity.
  /// * `slice` - The parent slice.
  /// * `x` - The local column.
  /// * `y` - The local row.
  /// * `canvas` - The destination canvas.
  pub fn render_on(
    &self,
    pool: &mut UiObjectPool,
    id: HyperlinkId,
    slice: SliceId,
    x: u16,
    y: u16,
    canvas: &mut CanvasService,
  ) -> bool {
    self.render_at(pool, id, Some(slice), (i32::from(x), i32::from(y)), canvas)
  }

  /// Render a link at its original signed base or slice coordinates and clip its hit region.
  ///
  /// # Arguments
  ///
  /// * `pool` - The owning pool.
  /// * `id` - The link identity.
  /// * `slice` - The parent slice, or `None` for the base.
  /// * `position` - The signed local column and row.
  /// * `canvas` - The destination canvas.
  pub fn render_at(
    &self,
    pool: &mut UiObjectPool,
    id: HyperlinkId,
    slice: Option<SliceId>,
    position: (i32, i32),
    canvas: &mut CanvasService,
  ) -> bool {
    let Some((text, style, hover_style)) = pool.hyperlinks.links.get(&id).map(|state| {
      (
        state.options.text.clone(),
        state.options.style.clone(),
        state.options.hover_style.clone(),
      )
    }) else {
      return false;
    };
    let mut params = draw_params(0, 0, text, style);
    let size = tg_service_layout::Size {
      width: text_width(&params),
      height: 1,
    };
    let resolved = match slice {
      None => canvas.base_hit_rect_at(position, size),
      Some(slice) => canvas.slice_hit_rect_at(slice, position, size),
    };
    self.render_resolved(pool, id, resolved);
    if self.is_hovered(pool, id)
      && let Some(style) = hover_style
    {
      params = draw_params(0, 0, params.text, style);
    }
    match slice {
      None => {
        canvas.text_at(position.0, position.1, &params);
        true
      }
      Some(slice) => canvas.text_at_on(slice, position.0, position.1, &params),
    }
  }

  /// Replace a link's text, target and styles, retaining its identity.
  ///
  /// # Arguments
  ///
  /// * `pool` - The owning pool.
  /// * `id` - The link identity.
  /// * `options` - The validated replacement configuration.
  pub fn set_options(
    &self,
    pool: &mut UiObjectPool,
    id: HyperlinkId,
    options: HyperlinkOptions,
  ) -> bool {
    if !validate_options(&options) {
      return false;
    }
    let Some(state) = pool.hyperlinks.links.get_mut(&id) else {
      return false;
    };
    if !options.mouse {
      if pool.hyperlinks.pressed == Some(id) {
        pool.hyperlinks.pressed = None;
      }
      if pool.hyperlinks.hovered == Some(id) {
        pool.hyperlinks.hovered = None;
      }
    }
    state.options = options;
    state.hit = None;
    true
  }

  /// Queue a click notification independently of the link's mouse option.
  pub fn click(&self, pool: &mut UiObjectPool, id: HyperlinkId) -> bool {
    let Some(link) = self.link(pool, id).map(str::to_owned) else {
      return false;
    };
    pool.push_hyperlink_event(HyperlinkEvent::Clicked { id, link });
    true
  }

  /// Report whether the latest pointer observation belongs to the link's visible hit region.
  pub fn is_hovered(&self, pool: &UiObjectPool, id: HyperlinkId) -> bool {
    pool.hyperlinks.hovered == Some(id)
      && pool.hyperlinks.pointer.is_some_and(|(x, y)| {
        pool
          .hyperlinks
          .links
          .get(&id)
          .and_then(|state| state.hit)
          .is_some_and(|hit| {
            hit.rect.contains(x, y) && pool.pointer_region_is_top(hit.surface_rank, hit.order, x, y)
          })
      })
  }

  /// Discard pointer state and stale clicks when the pool loses interaction ownership.
  pub fn deactivate(&self, pool: &mut UiObjectPool) {
    pool.hyperlinks.pressed = None;
    pool.hyperlinks.hovered = None;
    pool.hyperlinks.pointer = None;
    pool.hyperlinks.clear_hits();
    pool.events.retain(|event| event.hyperlink_id().is_none());
  }

  fn render_resolved(
    &self,
    pool: &mut UiObjectPool,
    id: HyperlinkId,
    resolved: Option<(Rect, (i32, i32), usize)>,
  ) -> bool {
    if !pool.hyperlinks.links.contains_key(&id) {
      return false;
    }
    let order = pool.next_render_order();
    let state = pool.hyperlinks.links.get_mut(&id).unwrap();
    state.hit = resolved
      .filter(|_| state.options.mouse)
      .map(|(rect, _, surface_rank)| HyperlinkHit {
        rect,
        order,
        surface_rank,
      });
    true
  }

  /// Route a pointer event through the component's current hit regions and focus state.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `_text_input` - The input service retained for existing callers; the pool resolves precedence.
  /// * `event` - The event to apply or route.
  pub fn route_mouse_event(
    &self,
    pool: &mut UiObjectPool,
    _text_input: &TextInputService,
    event: MouseEvent,
  ) -> bool {
    pool.hyperlinks.pointer = Some((event.x, event.y));
    pool.hyperlinks.hovered = pool
      .hyperlinks
      .hit(event.x, event.y)
      .filter(|(_, order)| pool.pointer_region_is_top(order.0, order.1, event.x, event.y))
      .map(|(id, _)| id);
    if pool
      .pointer_order_at(event.x, event.y)
      .is_some_and(|(_, order)| order == 0)
    {
      pool.hyperlinks.pressed = None;
      return true;
    }
    if !matches!(event.kind, MouseEventKind::Press | MouseEventKind::Release)
      || event.button != Some(MouseButton::Left)
    {
      return false;
    }
    let Some((id, _)) = pool
      .hyperlinks
      .hit(event.x, event.y)
      .filter(|(_, order)| pool.pointer_region_is_top(order.0, order.1, event.x, event.y))
    else {
      if matches!(event.kind, MouseEventKind::Press | MouseEventKind::Release) {
        pool.hyperlinks.pressed = None;
      }
      return false;
    };
    match event.kind {
      MouseEventKind::Press => {
        pool.hyperlinks.pressed = Some(id);
        true
      }
      MouseEventKind::Release => {
        let clicked = pool.hyperlinks.pressed.take() == Some(id);
        if clicked {
          self.click(pool, id);
        }
        true
      }
      _ => false,
    }
  }
}

fn validate_options(options: &HyperlinkOptions) -> bool {
  !options.link.is_empty() && !options.text.is_empty()
}

fn draw_params(x: u16, y: u16, text: String, style: TextStyle) -> DrawTextParams {
  DrawTextParams {
    x,
    y,
    text,
    fg: style.foreground,
    bg: style.background,
    bold: style.bold,
    italic: style.italic,
    underline: style.underline,
    strike: style.strike,
    blink: style.blink,
    reverse: style.reverse,
    hidden: style.hidden,
    dim: style.dim,
    wrap_mode: TextWrapMode::None,
    ..Default::default()
  }
}

fn text_width(params: &DrawTextParams) -> u16 {
  text_layout::measure_draw_text(params).0
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_validates_options() {
    let service = HyperlinkService::new();
    let mut pool = UiObjectPool::new();
    assert!(
      service
        .create(&mut pool, HyperlinkOptions::new("", "Open"))
        .is_none()
    );
    assert!(
      service
        .create(&mut pool, HyperlinkOptions::new("https://example.com", ""))
        .is_none()
    );
  }

  #[test]
  fn click_pushes_event() {
    let service = HyperlinkService::new();
    let text_input = TextInputService::new();
    let mut pool = UiObjectPool::new();
    let mut canvas = CanvasService::new();
    let id = service
      .create(
        &mut pool,
        HyperlinkOptions::new("https://example.com", "Example"),
      )
      .unwrap();

    assert!(service.render(&mut pool, id, 1, 1, &mut canvas));
    assert!(service.route_mouse_event(
      &mut pool,
      &text_input,
      MouseEvent {
        kind: MouseEventKind::Press,
        button: Some(MouseButton::Left),
        scroll: None,
        x: 2,
        y: 1,
      },
    ));
    assert!(service.route_mouse_event(
      &mut pool,
      &text_input,
      MouseEvent {
        kind: MouseEventKind::Release,
        button: Some(MouseButton::Left),
        scroll: None,
        x: 2,
        y: 1,
      },
    ));

    assert_eq!(
      pool.pop_event(),
      Some(crate::UiEvent::Hyperlink(HyperlinkEvent::Clicked {
        id,
        link: "https://example.com".to_string(),
      },)),
    );
  }

  #[test]
  fn render_supports_rich_text_and_base_style() {
    use tg_core_style::{TerminalColor, TextColor};

    let service = HyperlinkService::new();
    let mut pool = UiObjectPool::new();
    let mut canvas = CanvasService::new();
    let id = service
      .create(
        &mut pool,
        HyperlinkOptions::new("https://example.com", "f%<fg:red>Red</fg> Plain")
          .bg(TextColor::Terminal(TerminalColor::BrightBlack)),
      )
      .unwrap();

    assert!(service.render(&mut pool, id, 0, 0, &mut canvas));

    let first = canvas.cell_at(0, 0).unwrap();
    let plain = canvas.cell_at(4, 0).unwrap();
    assert_eq!(first.text, "R");
    assert_eq!(
      first.style.foreground,
      Some(TextColor::Terminal(TerminalColor::Red))
    );
    assert_eq!(
      plain.style.background,
      Some(TextColor::Terminal(TerminalColor::BrightBlack))
    );
    assert!(plain.style.underline);
  }
}
