//! Markdown layout, syntax-highlighted code, and clickable text regions.

mod layout;
mod state;
pub use layout::MarkdownLayout;
mod types;

pub(crate) use self::state::MarkdownViewObjects;
use self::state::{MarkdownLinkHit, MarkdownViewState};
pub use self::types::{
  MarkdownEvent, MarkdownRenderParams, MarkdownTheme, MarkdownViewId, MarkdownViewOptions,
};
use crate::UiObjectPool;
use crate::{ScrollBoxId, SliceId};
use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use tg_core_input::{MouseButton, MouseEvent, MouseEventKind};
use tg_core_style::{RichTextSegment, TextColor, TextStyle};
use tg_core_unicode::display_width;
use tg_service_canvas::CanvasService;
use tg_service_code_highlight::CodeHighlightService;
use tg_service_layout::{Rect, Size};
use tg_service_text_layout::TextAlign;
use tg_service_text_layout::{self as text_layout, DrawTextParams, TextWrapMode};

/// The public entry point for markdown operations.
#[derive(Default)]
pub struct MarkdownService;

#[derive(Clone, Copy)]
enum MarkdownTarget {
  Base,
  Slice(SliceId),
  ScrollBox(ScrollBoxId),
}

#[derive(Clone, Debug)]
enum MdBlock {
  Text {
    segments: Vec<RichTextSegment>,
    links: Vec<LinkSpan>,
    continuation: Vec<RichTextSegment>,
  },
  Code {
    language: Option<String>,
    code: String,
  },
  Rule,
  Table {
    rows: Vec<Vec<String>>,
    aligns: Vec<TextAlign>,
  },
  Blank,
}

#[derive(Clone, Debug)]
struct LinkSpan {
  start: usize,
  width: usize,
  href: String,
  text: String,
}

#[derive(Clone)]
struct InlineState {
  style_stack: Vec<TextStyle>,
  current: TextStyle,
  segments: Vec<RichTextSegment>,
  links: Vec<LinkSpan>,
  active_link: Option<ActiveLink>,
  continuation: Vec<RichTextSegment>,
}

#[derive(Clone)]
struct ActiveLink {
  href: String,
  start: usize,
  text: String,
}

struct ListState {
  ordered: bool,
  next: u64,
}

impl MarkdownService {
  /// Create a markdown service with its initial state.
  pub fn new() -> Self {
    Self
  }

  /// Create an owned markdown object and return its identity.
  pub fn create(
    &self,
    pool: &mut UiObjectPool,
    options: MarkdownViewOptions,
  ) -> Option<MarkdownViewId> {
    let id = MarkdownViewId(pool.markdown_views.next_id);
    pool.markdown_views.next_id += 1;
    pool.markdown_views.views.insert(
      id,
      MarkdownViewState {
        options,
        hits: Vec::new(),
        pressed: None,
      },
    );
    Some(id)
  }

  /// Remove the identified widget object and release its owned state.
  pub fn remove(&self, pool: &mut UiObjectPool, id: MarkdownViewId) -> bool {
    if pool.markdown_views.views.remove(&id).is_none() {
      return false;
    }
    pool.events.retain(|event| event.markdown_id() != Some(id));
    true
  }

  /// Report whether the identified widget object is still present.
  pub fn exists(&self, pool: &UiObjectPool, id: MarkdownViewId) -> bool {
    pool.markdown_views.views.contains_key(&id)
  }

  /// Return the source text retained by the identified Markdown view.
  pub fn markdown<'a>(&self, pool: &'a UiObjectPool, id: MarkdownViewId) -> Option<&'a str> {
    Some(&pool.markdown_views.views.get(&id)?.options.markdown)
  }

  /// Update the markdown used by this markdown service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `markdown` - The markdown.
  pub fn set_markdown(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    markdown: String,
  ) -> bool {
    let Some(state) = pool.markdown_views.views.get_mut(&id) else {
      return false;
    };
    state.hits.clear();
    state.pressed = None;
    state.options.markdown = markdown;
    true
  }

  /// Update the theme used by this markdown service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `theme` - The theme.
  pub fn set_theme(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    theme: MarkdownTheme,
  ) -> bool {
    let Some(state) = pool.markdown_views.views.get_mut(&id) else {
      return false;
    };
    state.options.theme = theme;
    true
  }

  /// Update the code theme used by this markdown service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `theme` - The theme.
  pub fn set_code_theme(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    theme: tg_service_code_highlight::CodeHighlightTheme,
  ) -> bool {
    let Some(state) = pool.markdown_views.views.get_mut(&id) else {
      return false;
    };
    state.options.code_theme = theme;
    true
  }

  /// Measure the laid-out Markdown content in terminal cells.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `width` - The width in terminal columns.
  /// * `code_highlight` - The code highlight.
  pub fn measure(
    &self,
    pool: &UiObjectPool,
    id: MarkdownViewId,
    width: u16,
    code_highlight: &CodeHighlightService,
  ) -> Option<Size> {
    let state = pool.markdown_views.views.get(&id)?;
    Some(
      self
        .layout(&state.options, width, false, code_highlight)
        .size(),
    )
  }

  /// Render the markdown service into its requested terminal-cell surface.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `params` - The formatting or rendering parameters.
  /// * `canvas` - The clipped canvas used for drawing.
  /// * `code_highlight` - The code highlight.
  pub fn render(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    params: MarkdownRenderParams,
    canvas: &mut CanvasService,
    code_highlight: &CodeHighlightService,
  ) -> bool {
    self.render_to(
      pool,
      id,
      params,
      MarkdownTarget::Base,
      canvas,
      code_highlight,
    )
  }

  /// Render the component into the identified clipped slice and update its interaction geometry.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `slice` - The slice.
  /// * `params` - The formatting or rendering parameters.
  /// * `canvas` - The clipped canvas used for drawing.
  /// * `code_highlight` - The code highlight.
  pub fn render_on(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    slice: SliceId,
    params: MarkdownRenderParams,
    canvas: &mut CanvasService,
    code_highlight: &CodeHighlightService,
  ) -> bool {
    self.render_to(
      pool,
      id,
      params,
      MarkdownTarget::Slice(slice),
      canvas,
      code_highlight,
    )
  }

  /// Render the component into the clipped scroll-box content viewport and update its interaction
  /// geometry.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `scroll_box` - The scroll box.
  /// * `params` - The formatting or rendering parameters.
  /// * `canvas` - The clipped canvas used for drawing.
  /// * `code_highlight` - The code highlight.
  pub fn render_in_scroll_box(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    scroll_box: ScrollBoxId,
    params: MarkdownRenderParams,
    canvas: &mut CanvasService,
    code_highlight: &CodeHighlightService,
  ) -> bool {
    self.render_to(
      pool,
      id,
      params,
      MarkdownTarget::ScrollBox(scroll_box),
      canvas,
      code_highlight,
    )
  }

  fn render_to(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    params: MarkdownRenderParams,
    target: MarkdownTarget,
    canvas: &mut CanvasService,
    code_highlight: &CodeHighlightService,
  ) -> bool {
    if params.width == 0 {
      return false;
    }
    let Some(options) = pool
      .markdown_views
      .views
      .get(&id)
      .map(|state| state.options.clone())
    else {
      return false;
    };
    let layout = self.layout(&options, params.width, false, code_highlight);
    layout::render_layout(pool, id, target, &layout, params, canvas)
  }

  /// Cancel pending link activation when the owning surface loses interaction ownership.
  pub fn deactivate(&self, pool: &mut UiObjectPool) {
    for state in pool.markdown_views.views.values_mut() {
      state.pressed = None;
    }
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
    _text_input: &crate::TextInputService,
    event: MouseEvent,
  ) -> bool {
    if pool
      .pointer_order_at(event.x, event.y)
      .is_some_and(|(_, order)| order == 0)
    {
      for state in pool.markdown_views.views.values_mut() {
        state.pressed = None;
      }
      return true;
    }
    if !matches!(event.kind, MouseEventKind::Press | MouseEventKind::Release)
      || event.button != Some(MouseButton::Left)
    {
      return false;
    }
    let Some((id, link_index, _)) = markdown_hit(pool, event.x, event.y) else {
      if event.kind == MouseEventKind::Release {
        for state in pool.markdown_views.views.values_mut() {
          state.pressed = None;
        }
      }
      return false;
    };
    match event.kind {
      MouseEventKind::Press => {
        self.deactivate(pool);
        if let Some(state) = pool.markdown_views.views.get_mut(&id) {
          let hit = &state.hits[link_index];
          state.pressed = Some((hit.href.clone(), hit.text.clone()));
        }
        true
      }
      MouseEventKind::Release => {
        let Some(state) = pool.markdown_views.views.get_mut(&id) else {
          return false;
        };
        let hit = &state.hits[link_index];
        let clicked = state.pressed.take() == Some((hit.href.clone(), hit.text.clone()));
        if clicked {
          let hit = state.hits[link_index].clone();
          pool.push_markdown_event(MarkdownEvent::LinkClicked {
            id,
            href: hit.href,
            text: hit.text,
          });
        }
        self.deactivate(pool);
        true
      }
      _ => false,
    }
  }
}

fn parse_markdown(
  options: &MarkdownViewOptions,
  _code_highlight: &CodeHighlightService,
) -> Vec<MdBlock> {
  let mut parser_options = Options::empty();
  parser_options.insert(Options::ENABLE_TABLES);
  parser_options.insert(Options::ENABLE_TASKLISTS);
  parser_options.insert(Options::ENABLE_STRIKETHROUGH);
  let parser = Parser::new_ext(&options.markdown, parser_options);

  let mut blocks = Vec::new();
  let mut inline: Option<InlineState> = None;
  let mut list_stack: Vec<ListState> = Vec::new();
  let mut quote_depth = 0usize;
  let mut code: Option<(Option<String>, String)> = None;
  let mut table_rows: Option<Vec<Vec<String>>> = None;
  let mut table_aligns = Vec::new();
  let mut table_row: Option<Vec<String>> = None;
  let mut table_cell = String::new();
  let mut in_table_cell = false;
  let mut ignore_image_depth = 0usize;

  for event in parser {
    if let Some((_, code_text)) = code.as_mut() {
      match event {
        Event::End(TagEnd::CodeBlock) => {
          let (language, code_text) = code.take().unwrap();
          blocks.push(MdBlock::Code {
            language,
            code: code_text,
          });
          blocks.push(MdBlock::Blank);
        }
        Event::Text(text) | Event::Code(text) | Event::Html(text) | Event::InlineHtml(text) => {
          code_text.push_str(&text)
        }
        Event::SoftBreak | Event::HardBreak => code_text.push('\n'),
        _ => {}
      }
      continue;
    }

    if ignore_image_depth > 0 {
      match event {
        Event::Start(Tag::Image { .. }) => ignore_image_depth += 1,
        Event::End(TagEnd::Image) => ignore_image_depth = ignore_image_depth.saturating_sub(1),
        _ => {}
      }
      continue;
    }

    match event {
      Event::Start(Tag::Heading { level, .. }) => {
        inline = Some(InlineState::new(heading_style(&options.theme, level)));
      }
      Event::End(TagEnd::Heading(_)) => finish_text(&mut blocks, inline.take(), true),
      Event::Start(Tag::Paragraph) => {
        if !list_stack.is_empty() && inline.is_some() {
          continue;
        }
        let mut state = InlineState::new(if quote_depth > 0 {
          options.theme.quote.clone()
        } else {
          options.theme.paragraph.clone()
        });
        for _ in 0..quote_depth {
          state.push_text(&options.theme.quote_marker, &options.theme.quote);
          state.continuation.push(RichTextSegment {
            text: options.theme.quote_marker.clone(),
            style: options.theme.quote.clone(),
          });
        }
        inline = Some(state);
      }
      Event::End(TagEnd::Paragraph) => finish_text(&mut blocks, inline.take(), true),
      Event::Start(Tag::BlockQuote(_)) => quote_depth += 1,
      Event::End(TagEnd::BlockQuote(_)) => {
        quote_depth = quote_depth.saturating_sub(1);
        blocks.push(MdBlock::Blank);
      }
      Event::Start(Tag::List(start)) => list_stack.push(ListState {
        ordered: start.is_some(),
        next: start.unwrap_or(1),
      }),
      Event::End(TagEnd::List(_)) => {
        list_stack.pop();
        blocks.push(MdBlock::Blank);
      }
      Event::Start(Tag::Item) => {
        let marker = list_stack
          .last_mut()
          .map(|list| {
            if list.ordered {
              let marker = format!("{}. ", list.next);
              list.next += 1;
              marker
            } else {
              "• ".to_string()
            }
          })
          .unwrap_or_else(|| "• ".to_string());
        let marker = format!(
          "{}{}",
          "  ".repeat(list_stack.len().saturating_sub(1)),
          marker
        );
        let marker_style = TextStyle {
          foreground: Some(TextColor::Rgb {
            r: 255,
            g: 255,
            b: 255,
          }),
          ..Default::default()
        };
        let mut state = InlineState::new(options.theme.paragraph.clone());
        state.push_text(&marker, &marker_style);
        state.continuation.push(RichTextSegment {
          text: " ".repeat(display_width(&marker)),
          style: marker_style,
        });
        inline = Some(state);
      }
      Event::End(TagEnd::Item) => finish_text(&mut blocks, inline.take(), false),
      Event::TaskListMarker(checked) => {
        if let Some(state) = inline.as_mut() {
          let marker = if checked { "[x] " } else { "[ ] " };
          let style = if checked {
            &options.theme.task_checked
          } else {
            &options.theme.task_unchecked
          };
          state.push_text(marker, style);
          state.continuation.push(RichTextSegment {
            text: " ".repeat(display_width(marker)),
            style: style.clone(),
          });
        }
      }
      Event::Start(Tag::CodeBlock(kind)) => {
        let language = match kind {
          CodeBlockKind::Fenced(info) => info.split_whitespace().next().map(str::to_string),
          CodeBlockKind::Indented => None,
        };
        code = Some((language, String::new()));
      }
      Event::Rule => {
        blocks.push(MdBlock::Rule);
        blocks.push(MdBlock::Blank);
      }
      Event::Start(Tag::Table(aligns)) => {
        table_aligns = aligns.into_iter().map(markdown_table_align).collect();
        table_rows = Some(Vec::new());
      }
      Event::End(TagEnd::Table) => {
        blocks.push(MdBlock::Table {
          rows: table_rows.take().unwrap_or_default(),
          aligns: std::mem::take(&mut table_aligns),
        });
        blocks.push(MdBlock::Blank);
      }
      Event::Start(Tag::TableRow) => table_row = Some(Vec::new()),
      Event::End(TagEnd::TableRow) => {
        if let (Some(rows), Some(row)) = (table_rows.as_mut(), table_row.take()) {
          rows.push(row);
        }
      }
      Event::Start(Tag::TableCell) => {
        table_cell.clear();
        in_table_cell = true;
      }
      Event::End(TagEnd::TableCell) => {
        in_table_cell = false;
        if let Some(row) = table_row.as_mut() {
          row.push(std::mem::take(&mut table_cell));
        }
      }
      Event::Start(Tag::Strong) => push_inline_style(&mut inline, &options.theme.bold),
      Event::End(TagEnd::Strong) => pop_inline_style(&mut inline),
      Event::Start(Tag::Emphasis) => push_inline_style(&mut inline, &options.theme.italic),
      Event::End(TagEnd::Emphasis) => pop_inline_style(&mut inline),
      Event::Start(Tag::Strikethrough) => push_inline_style(&mut inline, &options.theme.strike),
      Event::End(TagEnd::Strikethrough) => pop_inline_style(&mut inline),
      Event::Start(Tag::Link { dest_url, .. }) => {
        if let Some(state) = inline.as_mut() {
          state.start_link(dest_url.to_string(), &options.theme.link);
        }
      }
      Event::End(TagEnd::Link) => {
        if let Some(state) = inline.as_mut() {
          state.end_link();
        }
      }
      Event::Start(Tag::Image { .. }) => ignore_image_depth = 1,
      Event::Text(text) | Event::Html(text) | Event::InlineHtml(text) => {
        if in_table_cell {
          table_cell.push_str(&text);
        } else if let Some(state) = inline.as_mut() {
          state.push_current(&text);
        }
      }
      Event::Code(text) => {
        if in_table_cell {
          table_cell.push_str(&text);
        } else if let Some(state) = inline.as_mut() {
          state.push_text(&text, &options.theme.inline_code);
        }
      }
      Event::SoftBreak | Event::HardBreak => {
        if in_table_cell {
          table_cell.push(' ');
        } else if let Some(state) = inline.as_mut() {
          state.push_current("\n");
        }
      }
      _ => {}
    }
  }
  blocks
}

fn register_links(
  pool: &mut UiObjectPool,
  id: MarkdownViewId,
  links: &[LinkSpan],
  x: u16,
  y: u16,
  target: MarkdownTarget,
  canvas: &CanvasService,
) {
  for link in links {
    let Ok(start) = u16::try_from(link.start) else {
      continue;
    };
    let width = link.width.min(usize::from(u16::MAX - start)) as u16;
    if width == 0 {
      continue;
    }
    let rect = Rect {
      x: x.saturating_add(start),
      y,
      width,
      height: 1,
    };
    let resolved = match target {
      MarkdownTarget::Base => canvas.base_hit_rect(rect),
      MarkdownTarget::Slice(slice) => canvas.slice_hit_rect(slice, rect),
      MarkdownTarget::ScrollBox(scroll_box) => canvas.scroll_box_hit_rect(scroll_box, rect),
    };
    let Some((rect, _, surface_rank)) = resolved else {
      continue;
    };
    let order = pool.next_render_order();
    if let Some(state) = pool.markdown_views.views.get_mut(&id) {
      state.hits.push(MarkdownLinkHit {
        rect,
        order,
        surface_rank,
        href: link.href.clone(),
        text: link.text.clone(),
      });
    }
  }
}

fn markdown_hit(
  pool: &UiObjectPool,
  x: u16,
  y: u16,
) -> Option<(MarkdownViewId, usize, (usize, u64))> {
  pool
    .markdown_views
    .views
    .iter()
    .flat_map(|(id, state)| {
      state
        .hits
        .iter()
        .enumerate()
        .filter_map(move |(index, hit)| {
          hit
            .rect
            .contains(x, y)
            .then_some(())
            .filter(|_| pool.pointer_region_is_top(hit.surface_rank, hit.order, x, y))
            .map(|_| (*id, index, (hit.surface_rank, hit.order)))
        })
    })
    .max_by_key(|(_, _, order)| *order)
}

fn draw_segments(
  canvas: &mut CanvasService,
  target: MarkdownTarget,
  segments: &[RichTextSegment],
  params: &DrawTextParams,
) -> bool {
  match target {
    MarkdownTarget::Base => {
      canvas.rich_text_segments(segments, params);
      true
    }
    MarkdownTarget::Slice(slice) => canvas.rich_text_segments_on(slice, segments, params),
    MarkdownTarget::ScrollBox(scroll_box) => {
      canvas.rich_text_segments_in_scroll_box(scroll_box, segments, params)
    }
  }
}

fn finish_text(blocks: &mut Vec<MdBlock>, state: Option<InlineState>, blank_after: bool) {
  let Some(state) = state else {
    return;
  };
  if state
    .segments
    .iter()
    .any(|segment| !segment.text.is_empty())
  {
    blocks.push(MdBlock::Text {
      segments: state.segments,
      links: state.links,
      continuation: state.continuation,
    });
  }
  if blank_after {
    blocks.push(MdBlock::Blank);
  }
}

fn push_inline_style(inline: &mut Option<InlineState>, style: &TextStyle) {
  if let Some(state) = inline {
    state.push_style(style);
  }
}

fn pop_inline_style(inline: &mut Option<InlineState>) {
  if let Some(state) = inline {
    state.pop_style();
  }
}

impl InlineState {
  fn new(style: TextStyle) -> Self {
    Self {
      style_stack: vec![style.clone()],
      current: style,
      segments: Vec::new(),
      links: Vec::new(),
      active_link: None,
      continuation: Vec::new(),
    }
  }

  fn push_current(&mut self, text: &str) {
    self.push_text(text, &self.current.clone());
  }

  fn push_text(&mut self, text: &str, style: &TextStyle) {
    if text.is_empty() {
      return;
    }
    if let Some(link) = self.active_link.as_mut() {
      link.text.push_str(text);
    }
    self.segments.push(RichTextSegment {
      text: text.to_string(),
      style: style.clone(),
    });
  }

  fn push_style(&mut self, style: &TextStyle) {
    let merged = merge_style(&self.current, style);
    self.style_stack.push(merged.clone());
    self.current = merged;
  }

  fn pop_style(&mut self) {
    if self.style_stack.len() > 1 {
      self.style_stack.pop();
    }
    self.current = self.style_stack.last().cloned().unwrap_or_default();
  }

  fn start_link(&mut self, href: String, style: &TextStyle) {
    let start = self
      .segments
      .iter()
      .map(|segment| display_width(&segment.text))
      .sum();
    self.active_link = Some(ActiveLink {
      href,
      start,
      text: String::new(),
    });
    self.push_style(style);
  }

  fn end_link(&mut self) {
    self.pop_style();
    let Some(link) = self.active_link.take() else {
      return;
    };
    let width = display_width(&link.text);
    if width > 0 {
      self.links.push(LinkSpan {
        start: link.start,
        width,
        href: link.href,
        text: link.text,
      });
    }
  }
}

fn heading_style(theme: &MarkdownTheme, level: HeadingLevel) -> TextStyle {
  match level {
    HeadingLevel::H1 => theme.h1.clone(),
    HeadingLevel::H2 => theme.h2.clone(),
    HeadingLevel::H3 => theme.h3.clone(),
    HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => theme.h4_to_h6.clone(),
  }
}

fn merge_style(base: &TextStyle, overlay: &TextStyle) -> TextStyle {
  TextStyle {
    foreground: overlay
      .foreground
      .clone()
      .or_else(|| base.foreground.clone()),
    background: overlay
      .background
      .clone()
      .or_else(|| base.background.clone()),
    bold: base.bold || overlay.bold,
    italic: base.italic || overlay.italic,
    underline: base.underline || overlay.underline,
    strike: base.strike || overlay.strike,
    blink: base.blink || overlay.blink,
    reverse: base.reverse || overlay.reverse,
    hidden: base.hidden || overlay.hidden,
    dim: base.dim || overlay.dim,
  }
}

fn markdown_table_align(align: Alignment) -> TextAlign {
  match align {
    Alignment::Left | Alignment::None => TextAlign::Left,
    Alignment::Center => TextAlign::Center,
    Alignment::Right => TextAlign::Right,
  }
}

fn markdown_table_widths(rows: &[Vec<String>], width: u16, columns: usize) -> Vec<u16> {
  if columns == 0 {
    return Vec::new();
  }
  let available = (width as usize)
    .saturating_sub(columns.saturating_add(1))
    .max(columns);
  let mut widths = (0..columns)
    .map(|column| {
      rows
        .iter()
        .filter_map(|row| row.get(column))
        .map(|text| visible_width(text))
        .max()
        .unwrap_or(1)
        .max(1)
    })
    .collect::<Vec<_>>();
  let total = |values: &[u16]| values.iter().map(|value| *value as usize).sum::<usize>();
  if total(&widths) > available {
    let (mut low, mut high) = (1u16, *widths.iter().max().unwrap());
    while low < high {
      let middle = (u32::from(low) + u32::from(high)).div_ceil(2);
      if widths
        .iter()
        .map(|value| usize::from((*value).min(middle as u16)))
        .sum::<usize>()
        <= available
      {
        low = middle as u16;
      } else {
        high = middle as u16 - 1;
      }
    }
    for value in &mut widths {
      *value = (*value).min(low);
    }
  }
  let (mut low, mut high) = (1u16, width);
  while low < high {
    let middle = (u32::from(low) + u32::from(high)).div_ceil(2);
    if widths
      .iter()
      .map(|value| usize::from((*value).max(middle as u16)))
      .sum::<usize>()
      <= available
    {
      low = middle as u16;
    } else {
      high = middle as u16 - 1;
    }
  }
  for value in &mut widths {
    *value = (*value).max(low);
  }
  let remainder = available.saturating_sub(total(&widths));
  for value in widths.iter_mut().take(remainder) {
    *value = value.saturating_add(1);
  }
  widths
}

fn visible_width(text: &str) -> u16 {
  text_layout::measure_draw_text(&DrawTextParams {
    text: text.to_string(),
    max_height: Some(1),
    wrap_mode: TextWrapMode::None,
    ..Default::default()
  })
  .0
}

fn table_border(widths: &[u16], left: char, middle: char, right: char) -> String {
  let mut line = String::new();
  line.push(left);
  for (index, width) in widths.iter().enumerate() {
    line.push_str(&"─".repeat(*width as usize));
    line.push(if index + 1 == widths.len() {
      right
    } else {
      middle
    });
  }
  line
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn markdown_measure_handles_basic_blocks() {
    let service = MarkdownService::new();
    let code = CodeHighlightService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        MarkdownViewOptions::new("# Title\n\nHello **world**\n\n```rust\nfn main() {}\n```"),
      )
      .unwrap();
    let size = service.measure(&pool, id, 40, &code).unwrap();
    assert!(size.height > 3);
  }

  #[test]
  fn markdown_link_is_recorded_as_hit_after_render() {
    let service = MarkdownService::new();
    let code = CodeHighlightService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        MarkdownViewOptions::new("[Open](https://example.com)"),
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    let layout = tg_service_layout::LayoutService::new();
    canvas.begin_frame(&layout);
    pool.prepare_canvas(&mut canvas, &layout);
    assert!(service.render(
      &mut pool,
      id,
      MarkdownRenderParams {
        x: 0,
        y: 0,
        width: 30,
        max_height: None,
      },
      &mut canvas,
      &code,
    ));
    assert_eq!(pool.markdown_views.views[&id].hits.len(), 1);
  }

  #[test]
  fn markdown_images_are_ignored() {
    let code = CodeHighlightService::new();
    let options = MarkdownViewOptions::new("before ![alt text](image.png) after");
    let blocks = parse_markdown(&options, &code);
    let MdBlock::Text { segments, .. } = &blocks[0] else {
      panic!("expected text block");
    };
    let text = segments
      .iter()
      .map(|segment| segment.text.as_str())
      .collect::<String>();
    assert_eq!(text, "before  after");
  }

  #[test]
  fn markdown_click_survives_redraw_but_source_changes_cancel_it() {
    let service = MarkdownService::new();
    let code = CodeHighlightService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        MarkdownViewOptions::new("[Open](https://example.com)"),
      )
      .unwrap();
    let layout = tg_service_layout::LayoutService::new();
    let mut canvas = CanvasService::new();
    let params = MarkdownRenderParams {
      x: 0,
      y: 0,
      width: 30,
      max_height: None,
    };
    let input = crate::TextInputService::new();
    let mouse = |kind| MouseEvent {
      kind,
      x: 1,
      y: 0,
      button: Some(MouseButton::Left),
      scroll: None,
    };
    for replace in [false, true] {
      pool.begin_render();
      canvas.begin_frame(&layout);
      pool.prepare_canvas(&mut canvas, &layout);
      service.render(&mut pool, id, params, &mut canvas, &code);
      assert!(service.route_mouse_event(&mut pool, &input, mouse(MouseEventKind::Press)));
      if replace {
        service.set_markdown(&mut pool, id, "[Other](https://example.org)".into());
      }
      pool.begin_render();
      canvas.begin_frame(&layout);
      pool.prepare_canvas(&mut canvas, &layout);
      service.render(&mut pool, id, params, &mut canvas, &code);
      service.route_mouse_event(&mut pool, &input, mouse(MouseEventKind::Release));
      assert_eq!(pool.pop_event().is_some(), !replace);
    }
  }

  #[test]
  fn markdown_default_theme_uses_configured_rgb_styles() {
    let theme = MarkdownTheme::default();
    assert_style_color(&theme.h1, 170, 105, 225);
    assert_style_color(&theme.h2, 184, 122, 232);
    assert_style_color(&theme.h3, 198, 140, 238);
    assert_style_color(&theme.h4_to_h6, 234, 198, 250);
    assert_style_color(&theme.inline_code, 249, 232, 147);
    assert_style_color(&theme.quote, 255, 164, 209);
    assert_style_color(&theme.table_border, 255, 255, 255);
    assert_style_color(&theme.table_header, 86, 182, 194);
    assert_style_color(&theme.task_checked, 95, 215, 105);
    assert_style_color(&theme.task_unchecked, 85, 87, 83);
    assert!(theme.h1.bold);
    assert!(theme.h4_to_h6.bold);
  }

  #[test]
  fn markdown_list_markers_use_plain_white() {
    let code = CodeHighlightService::new();
    let options = MarkdownViewOptions::new("- item");
    let blocks = parse_markdown(&options, &code);
    let MdBlock::Text { segments, .. } = &blocks[0] else {
      panic!("expected text block");
    };
    assert_eq!(segments[0].text, "• ");
    assert_style_color(&segments[0].style, 255, 255, 255);
  }

  #[test]
  fn markdown_table_uses_white_border_and_cyan_header() {
    let service = MarkdownService::new();
    let code = CodeHighlightService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        MarkdownViewOptions::new("| A | B |\n|---|---|\n| 1 | 2 |"),
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    let layout = tg_service_layout::LayoutService::new();
    canvas.begin_frame(&layout);
    pool.prepare_canvas(&mut canvas, &layout);
    assert!(service.render(
      &mut pool,
      id,
      MarkdownRenderParams {
        x: 0,
        y: 0,
        width: 12,
        max_height: None,
      },
      &mut canvas,
      &code,
    ));
    assert_style_color(&canvas.cell_at(0, 0).unwrap().style, 255, 255, 255);
    assert_style_color(&canvas.cell_at(1, 1).unwrap().style, 86, 182, 194);
    assert_style_color(&canvas.cell_at(0, 1).unwrap().style, 255, 255, 255);
  }

  #[test]
  fn markdown_table_wraps_long_cells() {
    let service = MarkdownService::new();
    let code = CodeHighlightService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        MarkdownViewOptions::new(
          "| abc | dbe | ppppppppppp |\n|:--------|:----:|----:|\n|1|2|3|\n|4|5|ausgdhkjlasdhgiupwyitryqdhksdhfuipqweiuorqoiweyouiwer|",
        ),
      )
      .unwrap();
    let size = service.measure(&pool, id, 24, &code).unwrap();

    assert!(size.height > 7);
  }

  #[test]
  fn markdown_render_keeps_long_cjk_paragraph_complete() {
    let service = MarkdownService::new();
    let code = CodeHighlightService::new();
    let mut pool = UiObjectPool::new();
    let source = "当安全模式开启时，程序会限制部分高风险 API 的访问权限，以降低脚本对本地数据和系统环境造成影响的可能性。";
    let id = service
      .create(&mut pool, MarkdownViewOptions::new(source))
      .unwrap();
    let mut canvas = CanvasService::new();
    let layout = tg_service_layout::LayoutService::new();
    canvas.begin_frame(&layout);
    pool.prepare_canvas(&mut canvas, &layout);
    assert!(service.render(
      &mut pool,
      id,
      MarkdownRenderParams {
        x: 0,
        y: 0,
        width: 54,
        max_height: None,
      },
      &mut canvas,
      &code,
    ));

    let mut rendered = String::new();
    for y in 0..4 {
      for x in 0..54 {
        let Some(cell) = canvas.cell_at(x, y) else {
          continue;
        };
        if !cell.is_continuation() && cell.text != " " {
          rendered.push_str(&cell.text);
        }
      }
    }
    assert_eq!(rendered, source.replace(' ', ""));
  }

  fn assert_style_color(style: &TextStyle, r: u8, g: u8, b: u8) {
    assert_eq!(
      style.foreground,
      Some(TextColor::Rgb { r, g, b }),
      "{style:?}"
    );
  }
}
