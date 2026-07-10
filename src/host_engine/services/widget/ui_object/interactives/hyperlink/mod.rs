mod state;
mod types;

pub(crate) use self::state::HyperlinkObjects;
use self::state::{HyperlinkHit, HyperlinkState};
pub use self::types::{HyperlinkEvent, HyperlinkId, HyperlinkOptions};
use crate::host_engine::services::text_layout::{self, DrawTextParams, TextWrapMode};
use crate::host_engine::services::ui::UiObjectPool;
use crate::host_engine::services::{
  CanvasService, MouseButton, MouseEvent, MouseEventKind, Rect, SliceId, TextInputService,
  TextStyle,
};

pub struct HyperlinkService;

impl HyperlinkService {
  pub fn new() -> Self {
    Self
  }

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

  pub fn remove(&self, pool: &mut UiObjectPool, id: HyperlinkId) -> bool {
    if pool.hyperlinks.links.remove(&id).is_none() {
      return false;
    }
    if pool.hyperlinks.pressed == Some(id) {
      pool.hyperlinks.pressed = None;
    }
    pool.events.retain(|event| event.hyperlink_id() != Some(id));
    true
  }

  pub fn exists(&self, pool: &UiObjectPool, id: HyperlinkId) -> bool {
    pool.hyperlinks.links.contains_key(&id)
  }

  pub fn link<'a>(&self, pool: &'a UiObjectPool, id: HyperlinkId) -> Option<&'a str> {
    Some(&pool.hyperlinks.links.get(&id)?.options.link)
  }

  pub fn text<'a>(&self, pool: &'a UiObjectPool, id: HyperlinkId) -> Option<&'a str> {
    Some(&pool.hyperlinks.links.get(&id)?.options.text)
  }

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

  pub fn render(
    &self,
    pool: &mut UiObjectPool,
    id: HyperlinkId,
    x: u16,
    y: u16,
    canvas: &mut CanvasService,
  ) -> bool {
    let Some((text, style)) = pool
      .hyperlinks
      .links
      .get(&id)
      .map(|state| (state.options.text.clone(), state.options.style.clone()))
    else {
      return false;
    };
    let params = draw_params(x, y, text, style);
    canvas.text(&params);
    let width = text_width(&params);
    self.render_resolved(
      pool,
      id,
      canvas.base_hit_rect(Rect {
        x,
        y,
        width,
        height: 1,
      }),
    )
  }

  pub fn render_on(
    &self,
    pool: &mut UiObjectPool,
    id: HyperlinkId,
    slice: SliceId,
    x: u16,
    y: u16,
    canvas: &mut CanvasService,
  ) -> bool {
    let Some((text, style)) = pool
      .hyperlinks
      .links
      .get(&id)
      .map(|state| (state.options.text.clone(), state.options.style.clone()))
    else {
      return false;
    };
    let params = draw_params(x, y, text, style);
    if !canvas.text_on(slice, &params) {
      return false;
    }
    let width = text_width(&params);
    self.render_resolved(
      pool,
      id,
      canvas.slice_hit_rect(
        slice,
        Rect {
          x,
          y,
          width,
          height: 1,
        },
      ),
    )
  }

  pub(crate) fn render_host(
    &self,
    pool: &mut UiObjectPool,
    id: HyperlinkId,
    x: u16,
    y: u16,
    canvas: &mut CanvasService,
  ) -> bool {
    let Some((text, style)) = pool
      .hyperlinks
      .links
      .get(&id)
      .map(|state| (state.options.text.clone(), state.options.style.clone()))
    else {
      return false;
    };
    let params = draw_params(x, y, text, style);
    canvas.host_text(&params);
    let width = text_width(&params);
    self.render_resolved(
      pool,
      id,
      canvas.host_hit_rect(Rect {
        x,
        y,
        width,
        height: 1,
      }),
    )
  }

  fn render_resolved(
    &self,
    pool: &mut UiObjectPool,
    id: HyperlinkId,
    resolved: Option<(Rect, (u16, u16), usize)>,
  ) -> bool {
    if !pool.hyperlinks.links.contains_key(&id) {
      return false;
    }
    let order = pool.next_render_order();
    let state = pool.hyperlinks.links.get_mut(&id).unwrap();
    state.hit = resolved.map(|(rect, _, surface_rank)| HyperlinkHit {
      rect,
      order,
      surface_rank,
    });
    true
  }

  pub(crate) fn route_mouse_event(
    &self,
    pool: &mut UiObjectPool,
    text_input: &TextInputService,
    event: MouseEvent,
  ) -> bool {
    if !matches!(event.kind, MouseEventKind::Press | MouseEventKind::Release) {
      return false;
    }
    if event.button != Some(MouseButton::Left) {
      return false;
    }

    let Some((id, order)) = pool.hyperlinks.hit(event.x, event.y) else {
      if event.kind == MouseEventKind::Release {
        pool.hyperlinks.pressed = None;
      }
      return false;
    };
    if pool
      .hit_areas
      .hit(event.x, event.y)
      .is_some_and(|(_, hit_order)| hit_order > order)
      || text_input
        .mouse_hit_order(pool, event.x, event.y)
        .is_some_and(|text_order| text_order > order)
    {
      return false;
    }

    match event.kind {
      MouseEventKind::Press => {
        pool.hyperlinks.pressed = Some(id);
        true
      }
      MouseEventKind::Release => {
        let clicked = pool.hyperlinks.pressed.take() == Some(id);
        if clicked {
          let link = pool.hyperlinks.links[&id].options.link.clone();
          pool.push_hyperlink_event(HyperlinkEvent::Clicked { id, link });
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
      Some(crate::host_engine::services::UiEvent::Hyperlink(
        HyperlinkEvent::Clicked {
          id,
          link: "https://example.com".to_string(),
        },
      )),
    );
  }

  #[test]
  fn render_supports_rich_text_and_base_style() {
    use crate::host_engine::services::{TerminalColor, TextColor};

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
